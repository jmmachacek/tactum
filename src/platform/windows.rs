use std::{ffi::c_void, mem::size_of, thread, time::Duration};

use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};

use windows_sys::Win32::{
    Foundation::{BOOL, GlobalFree, HGLOBAL, LPARAM, RECT},
    Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CAPTUREBLT, CreateCompatibleDC,
        CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, EnumDisplayMonitors, GetDC,
        GetMonitorInfoW, HDC, HGDIOBJ, HMONITOR, MONITORINFO, RGBQUAD, ReleaseDC, SRCCOPY,
        SelectObject,
    },
    System::{
        DataExchange::{
            CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable,
            OpenClipboard, SetClipboardData,
        },
        Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock},
    },
    UI::{
        HiDpi::{
            DPI_AWARENESS_CONTEXT, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            SetThreadDpiAwarenessContext,
        },
        Input::KeyboardAndMouse::{
            INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
            KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, KEYEVENTF_UNICODE, MOUSE_EVENT_FLAGS,
            MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN,
            MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL,
            MOUSEINPUT, SendInput,
        },
        WindowsAndMessaging::{MONITORINFOF_PRIMARY, SetCursorPos, WHEEL_DELTA},
    },
};

use crate::{
    Error, Key, MouseButton, Point, Result,
    platform::{CapturedDisplay, CapturedScreenshot},
};

pub(crate) struct Computer;

const INPUT_EVENT_DELAY: Duration = Duration::from_millis(30);
const CF_UNICODETEXT: u32 = 13;

struct OpenClipboardGuard;

struct DpiAwarenessGuard(DPI_AWARENESS_CONTEXT);

impl DpiAwarenessGuard {
    fn enter() -> Result<Self> {
        let previous =
            unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        if previous.is_null() {
            return Err(Error::OperationFailed);
        }
        Ok(Self(previous))
    }
}

impl Drop for DpiAwarenessGuard {
    fn drop(&mut self) {
        unsafe { SetThreadDpiAwarenessContext(self.0) };
    }
}

impl OpenClipboardGuard {
    fn open() -> Result<Self> {
        for _ in 0..10 {
            if unsafe { OpenClipboard(std::ptr::null_mut()) } != 0 {
                return Ok(Self);
            }
            thread::sleep(Duration::from_millis(10));
        }
        Err(Error::OperationFailed)
    }
}

impl Drop for OpenClipboardGuard {
    fn drop(&mut self) {
        unsafe { CloseClipboard() };
    }
}

struct DisplayCollector {
    displays: Vec<CapturedDisplay>,
    primary_display: Option<u64>,
    failed: bool,
}

fn mouse_events(button: MouseButton) -> (MOUSE_EVENT_FLAGS, MOUSE_EVENT_FLAGS) {
    match button {
        MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
    }
}

fn mouse_input(flags: MOUSE_EVENT_FLAGS) -> INPUT {
    mouse_input_with_data(flags, 0)
}

fn mouse_input_with_data(flags: MOUSE_EVENT_FLAGS, mouse_data: u32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: mouse_data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn key_scan_code(key: Key) -> (u16, bool) {
    match key {
        Key::A => (0x1e, false),
        Key::B => (0x30, false),
        Key::C => (0x2e, false),
        Key::D => (0x20, false),
        Key::E => (0x12, false),
        Key::F => (0x21, false),
        Key::G => (0x22, false),
        Key::H => (0x23, false),
        Key::I => (0x17, false),
        Key::J => (0x24, false),
        Key::K => (0x25, false),
        Key::L => (0x26, false),
        Key::M => (0x32, false),
        Key::N => (0x31, false),
        Key::O => (0x18, false),
        Key::P => (0x19, false),
        Key::Q => (0x10, false),
        Key::R => (0x13, false),
        Key::S => (0x1f, false),
        Key::T => (0x14, false),
        Key::U => (0x16, false),
        Key::V => (0x2f, false),
        Key::W => (0x11, false),
        Key::X => (0x2d, false),
        Key::Y => (0x15, false),
        Key::Z => (0x2c, false),
        Key::Backspace => (0x0e, false),
        Key::Tab => (0x0f, false),
        Key::Return => (0x1c, false),
        Key::Escape => (0x01, false),
        Key::Space => (0x39, false),
        Key::Delete => (0x53, true),
        Key::Home => (0x47, true),
        Key::End => (0x4f, true),
        Key::PageUp => (0x49, true),
        Key::PageDown => (0x51, true),
        Key::Left => (0x4b, true),
        Key::Right => (0x4d, true),
        Key::Down => (0x50, true),
        Key::Up => (0x48, true),
        Key::Shift => (0x2a, false),
        Key::Control => (0x1d, false),
        Key::Option => (0x38, false),
        Key::Command => (0x5b, true),
    }
}

fn key_input(key: Key, key_down: bool) -> INPUT {
    let (scan_code, extended) = key_scan_code(key);
    let mut flags = KEYEVENTF_SCANCODE;
    if extended {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if !key_down {
        flags |= KEYEVENTF_KEYUP;
    }

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: scan_code,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn unicode_input(code_unit: u16, key_down: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: code_unit,
                dwFlags: if key_down {
                    KEYEVENTF_UNICODE
                } else {
                    KEYEVENTF_UNICODE | KEYEVENTF_KEYUP
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn cursor_position(point: Point) -> Result<(i32, i32)> {
    if point.x() < i32::MIN as f64
        || point.x() > i32::MAX as f64
        || point.y() < i32::MIN as f64
        || point.y() > i32::MAX as f64
    {
        return Err(Error::InvalidPoint);
    }

    Ok((point.x().round() as i32, point.y().round() as i32))
}

fn set_cursor_position((x, y): (i32, i32)) -> Result<()> {
    let _dpi_awareness = DpiAwarenessGuard::enter()?;
    if unsafe { SetCursorPos(x, y) } == 0 {
        return Err(Error::OperationFailed);
    }
    Ok(())
}

fn move_cursor(point: Point) -> Result<()> {
    set_cursor_position(cursor_position(point)?)
}

fn send_inputs(inputs: &[INPUT]) -> Result<()> {
    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        )
    };
    if sent != inputs.len() as u32 {
        return Err(Error::OperationFailed);
    }
    Ok(())
}

unsafe extern "system" fn collect_display(
    monitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let collector = unsafe { &mut *(data as *mut DisplayCollector) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        rcMonitor: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        rcWork: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        dwFlags: 0,
    };
    if unsafe { GetMonitorInfoW(monitor, &mut info) } == 0 {
        collector.failed = true;
        return 0;
    }

    let bounds = info.rcMonitor;
    let width = bounds.right - bounds.left;
    let height = bounds.bottom - bounds.top;
    if width <= 0 || height <= 0 {
        collector.failed = true;
        return 0;
    }

    collector.displays.push(CapturedDisplay {
        id: monitor as usize as u64,
        origin: Point::new(bounds.left as f64, bounds.top as f64)
            .expect("Windows monitor coordinates are finite"),
        width: width as f64,
        height: height as f64,
        scale_x: 1.0,
        scale_y: 1.0,
    });
    if info.dwFlags & MONITORINFOF_PRIMARY != 0 {
        collector.primary_display = Some(monitor as usize as u64);
    }
    1
}

fn enumerate_displays() -> Result<(Vec<CapturedDisplay>, u64)> {
    let _dpi_awareness = DpiAwarenessGuard::enter()?;
    let mut collector = DisplayCollector {
        displays: Vec::new(),
        primary_display: None,
        failed: false,
    };
    let succeeded = unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(collect_display),
            (&mut collector as *mut DisplayCollector) as LPARAM,
        )
    } != 0;

    if !succeeded || collector.failed {
        return Err(Error::OperationFailed);
    }

    let primary_display = collector.primary_display.ok_or(Error::OperationFailed)?;
    Ok((collector.displays, primary_display))
}

fn capture_rectangle(left: i32, top: i32, width: i32, height: i32) -> Result<CapturedScreenshot> {
    let _dpi_awareness = DpiAwarenessGuard::enter()?;
    if width <= 0 || height <= 0 {
        return Err(Error::OperationFailed);
    }

    let width_u32 = width as u32;
    let height_u32 = height as u32;
    let byte_len = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(Error::OperationFailed)?;
    if byte_len > u32::MAX as usize {
        return Err(Error::OperationFailed);
    }

    let screen = unsafe { GetDC(std::ptr::null_mut()) };
    if screen.is_null() {
        return Err(Error::OperationFailed);
    }
    let memory = unsafe { CreateCompatibleDC(screen) };
    if memory.is_null() {
        unsafe { ReleaseDC(std::ptr::null_mut(), screen) };
        return Err(Error::OperationFailed);
    }

    let bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: byte_len as u32,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD {
            rgbBlue: 0,
            rgbGreen: 0,
            rgbRed: 0,
            rgbReserved: 0,
        }],
    };

    let mut bits: *mut c_void = std::ptr::null_mut();
    let bitmap = unsafe {
        CreateDIBSection(
            screen,
            &bitmap_info,
            DIB_RGB_COLORS,
            &mut bits,
            std::ptr::null_mut(),
            0,
        )
    };
    if bitmap.is_null() || bits.is_null() {
        unsafe {
            if !bitmap.is_null() {
                DeleteObject(bitmap as HGDIOBJ);
            }
            DeleteDC(memory);
            ReleaseDC(std::ptr::null_mut(), screen);
        }
        return Err(Error::OperationFailed);
    }

    let previous = unsafe { SelectObject(memory, bitmap as HGDIOBJ) };
    if previous.is_null() {
        unsafe {
            DeleteObject(bitmap as HGDIOBJ);
            DeleteDC(memory);
            ReleaseDC(std::ptr::null_mut(), screen);
        }
        return Err(Error::OperationFailed);
    }

    let copied = unsafe {
        BitBlt(
            memory,
            0,
            0,
            width,
            height,
            screen,
            left,
            top,
            SRCCOPY | CAPTUREBLT,
        )
    };
    let mut rgba = if copied != 0 {
        unsafe { std::slice::from_raw_parts(bits.cast::<u8>(), byte_len) }.to_vec()
    } else {
        Vec::new()
    };
    unsafe {
        SelectObject(memory, previous);
        DeleteObject(bitmap as HGDIOBJ);
        DeleteDC(memory);
        ReleaseDC(std::ptr::null_mut(), screen);
    }
    if copied == 0 {
        return Err(Error::OperationFailed);
    }

    for pixel in rgba.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
    }
    let mut png = Vec::new();
    PngEncoder::new(&mut png)
        .write_image(&rgba, width_u32, height_u32, ExtendedColorType::Rgba8)
        .map_err(|_| Error::OperationFailed)?;

    Ok(CapturedScreenshot {
        png,
        width: width_u32,
        height: height_u32,
        desktop_origin: Point::new(left as f64, top as f64).map_err(|_| Error::OperationFailed)?,
        desktop_width: width as f64,
        desktop_height: height as f64,
    })
}

impl Computer {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self)
    }

    pub(crate) fn click(&self, button: MouseButton, point: Point) -> Result<()> {
        move_cursor(point)?;
        let (down, up) = mouse_events(button);
        send_inputs(&[mouse_input(down), mouse_input(up)])
    }

    pub(crate) fn double_click(&self, button: MouseButton, point: Point) -> Result<()> {
        move_cursor(point)?;
        let (down, up) = mouse_events(button);
        send_inputs(&[
            mouse_input(down),
            mouse_input(up),
            mouse_input(down),
            mouse_input(up),
        ])
    }

    pub(crate) fn move_to(&self, point: Point) -> Result<()> {
        move_cursor(point)
    }

    pub(crate) fn drag(&self, button: MouseButton, from: Point, to: Point) -> Result<()> {
        let from = cursor_position(from)?;
        let to = cursor_position(to)?;
        let (down, up) = mouse_events(button);

        set_cursor_position(from)?;
        send_inputs(&[mouse_input(down)])?;
        thread::sleep(INPUT_EVENT_DELAY);
        if let Err(error) = set_cursor_position(to) {
            let _ = send_inputs(&[mouse_input(up)]);
            return Err(error);
        }
        thread::sleep(INPUT_EVENT_DELAY);
        send_inputs(&[mouse_input(up)])
    }

    pub(crate) fn mouse_down(&self, button: MouseButton, point: Point) -> Result<()> {
        move_cursor(point)?;
        let (down, _) = mouse_events(button);
        send_inputs(&[mouse_input(down)])
    }

    pub(crate) fn mouse_up(&self, button: MouseButton, point: Point) -> Result<()> {
        move_cursor(point)?;
        let (_, up) = mouse_events(button);
        send_inputs(&[mouse_input(up)])
    }

    pub(crate) fn scroll(&self, horizontal: i32, vertical: i32) -> Result<()> {
        let mut inputs = Vec::with_capacity(2);
        if vertical != 0 {
            let delta = vertical
                .checked_mul(WHEEL_DELTA as i32)
                .ok_or(Error::OperationFailed)?;
            inputs.push(mouse_input_with_data(MOUSEEVENTF_WHEEL, delta as u32));
        }
        if horizontal != 0 {
            let delta = horizontal
                .checked_mul(WHEEL_DELTA as i32)
                .ok_or(Error::OperationFailed)?;
            inputs.push(mouse_input_with_data(MOUSEEVENTF_HWHEEL, delta as u32));
        }

        if inputs.is_empty() {
            Ok(())
        } else {
            send_inputs(&inputs)
        }
    }

    pub(crate) fn key_press(&self, key: Key) -> Result<()> {
        send_inputs(&[key_input(key, true), key_input(key, false)])
    }

    pub(crate) fn key_down(&self, key: Key) -> Result<()> {
        send_inputs(&[key_input(key, true)])
    }

    pub(crate) fn key_up(&self, key: Key) -> Result<()> {
        send_inputs(&[key_input(key, false)])
    }

    pub(crate) fn type_text(&self, text: &str) -> Result<()> {
        for character in text.chars() {
            match character {
                '\n' | '\r' => self.key_press(Key::Return)?,
                '\t' => self.key_press(Key::Tab)?,
                _ => {
                    let mut utf16 = [0; 2];
                    for &code_unit in character.encode_utf16(&mut utf16).iter() {
                        send_inputs(&[
                            unicode_input(code_unit, true),
                            unicode_input(code_unit, false),
                        ])?;
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn read_clipboard(&self) -> Result<Option<String>> {
        let _clipboard = OpenClipboardGuard::open()?;
        if unsafe { IsClipboardFormatAvailable(CF_UNICODETEXT) } == 0 {
            return Ok(None);
        }

        let memory = unsafe { GetClipboardData(CF_UNICODETEXT) } as HGLOBAL;
        if memory.is_null() {
            return Err(Error::OperationFailed);
        }
        let byte_len = unsafe { GlobalSize(memory) };
        if byte_len < size_of::<u16>() || byte_len % size_of::<u16>() != 0 {
            return Err(Error::OperationFailed);
        }
        let data = unsafe { GlobalLock(memory) }.cast::<u16>();
        if data.is_null() {
            return Err(Error::OperationFailed);
        }

        let code_units = unsafe { std::slice::from_raw_parts(data, byte_len / size_of::<u16>()) };
        let text = code_units
            .iter()
            .position(|&code_unit| code_unit == 0)
            .ok_or(Error::OperationFailed)
            .and_then(|end| {
                String::from_utf16(&code_units[..end]).map_err(|_| Error::OperationFailed)
            });

        unsafe { GlobalUnlock(memory) };

        text.map(Some)
    }

    pub(crate) fn write_clipboard(&self, text: &str) -> Result<()> {
        if text.contains('\0') {
            return Err(Error::OperationFailed);
        }

        let utf16: Vec<_> = text.encode_utf16().chain([0]).collect();
        let byte_len = utf16
            .len()
            .checked_mul(size_of::<u16>())
            .ok_or(Error::OperationFailed)?;
        let memory = unsafe { GlobalAlloc(GMEM_MOVEABLE, byte_len) };
        if memory.is_null() {
            return Err(Error::OperationFailed);
        }
        let data = unsafe { GlobalLock(memory) }.cast::<u16>();
        if data.is_null() {
            unsafe { GlobalFree(memory) };
            return Err(Error::OperationFailed);
        }

        unsafe {
            std::ptr::copy_nonoverlapping(utf16.as_ptr(), data, utf16.len());
            GlobalUnlock(memory);
        }

        let result = (|| {
            let _clipboard = OpenClipboardGuard::open()?;
            if unsafe { EmptyClipboard() } == 0 {
                return Err(Error::OperationFailed);
            }
            if unsafe { SetClipboardData(CF_UNICODETEXT, memory) }.is_null() {
                return Err(Error::OperationFailed);
            }
            Ok(())
        })();

        if result.is_err() {
            unsafe { GlobalFree(memory) };
        }

        result
    }

    pub(crate) fn displays(&self) -> Result<Vec<CapturedDisplay>> {
        Ok(enumerate_displays()?.0)
    }

    pub(crate) fn screenshot(&self) -> Result<CapturedScreenshot> {
        let (displays, primary_display) = enumerate_displays()?;
        let display = displays
            .into_iter()
            .find(|display| display.id == primary_display)
            .ok_or(Error::OperationFailed)?;

        capture_rectangle(
            display.origin.x() as i32,
            display.origin.y() as i32,
            display.width as i32,
            display.height as i32,
        )
    }

    pub(crate) fn screenshot_display(&self, id: u64) -> Result<CapturedScreenshot> {
        let display = self
            .displays()?
            .into_iter()
            .find(|display| display.id == id)
            .ok_or(Error::OperationFailed)?;

        capture_rectangle(
            display.origin.x() as i32,
            display.origin.y() as i32,
            display.width as i32,
            display.height as i32,
        )
    }

    pub(crate) fn screenshot_all_displays(&self) -> Result<CapturedScreenshot> {
        let displays = self.displays()?;
        let first = displays.first().ok_or(Error::OperationFailed)?;
        let (left, top, right, bottom) = displays.iter().skip(1).fold(
            (
                first.origin.x() as i32,
                first.origin.y() as i32,
                (first.origin.x() + first.width) as i32,
                (first.origin.y() + first.height) as i32,
            ),
            |(left, top, right, bottom), display| {
                (
                    left.min(display.origin.x() as i32),
                    top.min(display.origin.y() as i32),
                    right.max((display.origin.x() + display.width) as i32),
                    bottom.max((display.origin.y() + display.height) as i32),
                )
            },
        );

        capture_rectangle(
            left,
            top,
            right.checked_sub(left).ok_or(Error::OperationFailed)?,
            bottom.checked_sub(top).ok_or(Error::OperationFailed)?,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, sync::Mutex};

    use image::GenericImageView;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        INPUT_KEYBOARD, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE,
        KEYEVENTF_UNICODE,
    };

    use super::{
        Computer, EmptyClipboard, Key, OpenClipboardGuard, enumerate_displays, key_input,
        unicode_input,
    };
    use crate::platform::{CapturedDisplay, CapturedScreenshot};

    static CLIPBOARD_TEST: Mutex<()> = Mutex::new(());

    struct ClipboardRestore {
        original: Option<String>,
    }

    impl Drop for ClipboardRestore {
        fn drop(&mut self) {
            let computer = Computer;
            if let Some(text) = &self.original {
                let _ = computer.write_clipboard(text);
            } else if let Ok(_clipboard) = OpenClipboardGuard::open() {
                unsafe { EmptyClipboard() };
            }
        }
    }

    fn assert_valid_png(screenshot: &CapturedScreenshot) {
        let image = image::load_from_memory(&screenshot.png).unwrap();
        assert_eq!(image.dimensions(), (screenshot.width, screenshot.height));
        assert_eq!(&screenshot.png[..8], b"\x89PNG\r\n\x1a\n");
    }

    fn assert_matches_display(screenshot: &CapturedScreenshot, display: &CapturedDisplay) {
        assert_eq!(screenshot.desktop_origin, display.origin);
        assert_eq!(screenshot.desktop_width, display.width);
        assert_eq!(screenshot.desktop_height, display.height);
        assert_valid_png(screenshot);
    }

    #[test]
    fn enumerates_active_displays() {
        let (displays, primary_display) = enumerate_displays().unwrap();
        let display_ids: HashSet<_> = displays.iter().map(|display| display.id).collect();

        assert!(!displays.is_empty());
        assert!(displays.iter().any(|display| display.id == primary_display));
        assert_eq!(display_ids.len(), displays.len());
        for display in displays {
            assert!(display.width > 0.0);
            assert!(display.height > 0.0);
            assert_eq!(display.scale_x, 1.0);
            assert_eq!(display.scale_y, 1.0);
        }
    }

    #[test]
    fn captures_primary_selected_and_virtual_desktop() {
        let computer = Computer;
        let (displays, primary_display) = enumerate_displays().unwrap();
        let primary = displays
            .iter()
            .find(|display| display.id == primary_display)
            .unwrap();

        let screenshot = computer.screenshot().unwrap();
        assert_matches_display(&screenshot, primary);

        let selected = computer.screenshot_display(displays[0].id).unwrap();
        assert_matches_display(&selected, &displays[0]);

        let all = computer.screenshot_all_displays().unwrap();
        let first = &displays[0];
        let (min_x, min_y, max_x, max_y) = displays.iter().skip(1).fold(
            (
                first.origin.x(),
                first.origin.y(),
                first.origin.x() + first.width,
                first.origin.y() + first.height,
            ),
            |(min_x, min_y, max_x, max_y), display| {
                (
                    min_x.min(display.origin.x()),
                    min_y.min(display.origin.y()),
                    max_x.max(display.origin.x() + display.width),
                    max_y.max(display.origin.y() + display.height),
                )
            },
        );
        assert_eq!(all.desktop_origin.x(), min_x);
        assert_eq!(all.desktop_origin.y(), min_y);
        assert_eq!(all.desktop_width, max_x - min_x);
        assert_eq!(all.desktop_height, max_y - min_y);
        assert_valid_png(&all);
    }

    #[test]
    fn clipboard_round_trips_unicode_text() {
        let _lock = CLIPBOARD_TEST.lock().unwrap();
        let computer = Computer;
        let _restore = ClipboardRestore {
            original: computer.read_clipboard().unwrap(),
        };
        let text = "Tactum clipboard test\n世界 🚀";

        computer.write_clipboard(text).unwrap();
        assert_eq!(computer.read_clipboard().unwrap().as_deref(), Some(text));
    }

    #[test]
    fn creates_scan_code_and_unicode_keyboard_inputs() {
        let delete = key_input(Key::Delete, false);
        assert_eq!(delete.r#type, INPUT_KEYBOARD);
        let delete = unsafe { delete.Anonymous.ki };
        assert_eq!(delete.wScan, 0x53);
        assert_eq!(
            delete.dwFlags,
            KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP
        );

        let unicode = unicode_input('界' as u16, true);
        assert_eq!(unicode.r#type, INPUT_KEYBOARD);
        let unicode = unsafe { unicode.Anonymous.ki };
        assert_eq!(unicode.wScan, '界' as u16);
        assert_eq!(unicode.dwFlags, KEYEVENTF_UNICODE);
    }
}
