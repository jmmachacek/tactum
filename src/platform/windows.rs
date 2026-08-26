use std::{ffi::c_void, mem::size_of};

use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};

use windows_sys::Win32::{
    Foundation::{BOOL, LPARAM, RECT},
    Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CAPTUREBLT, CreateCompatibleDC,
        CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, EnumDisplayMonitors, GetDC,
        GetMonitorInfoW, HDC, HGDIOBJ, HMONITOR, MONITORINFO, RGBQUAD, ReleaseDC, SRCCOPY,
        SelectObject,
    },
    UI::{
        Input::KeyboardAndMouse::{
            INPUT, INPUT_0, INPUT_MOUSE, MOUSE_EVENT_FLAGS, MOUSEEVENTF_LEFTDOWN,
            MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
            MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT, SendInput,
        },
        WindowsAndMessaging::{MONITORINFOF_PRIMARY, SetCursorPos},
    },
};

use crate::{
    Error, Key, MouseButton, Point, Result,
    platform::{CapturedDisplay, CapturedScreenshot},
};

pub(crate) struct Computer;

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
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
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
        if point.x() < i32::MIN as f64
            || point.x() > i32::MAX as f64
            || point.y() < i32::MIN as f64
            || point.y() > i32::MAX as f64
        {
            return Err(Error::InvalidPoint);
        }

        let x = point.x().round() as i32;
        let y = point.y().round() as i32;
        if unsafe { SetCursorPos(x, y) } == 0 {
            return Err(Error::OperationFailed);
        }

        let (down, up) = mouse_events(button);
        let inputs = [mouse_input(down), mouse_input(up)];
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

    pub(crate) fn double_click(&self, _button: MouseButton, _point: Point) -> Result<()> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn move_to(&self, _point: Point) -> Result<()> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn drag(&self, _button: MouseButton, _from: Point, _to: Point) -> Result<()> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn mouse_down(&self, _button: MouseButton, _point: Point) -> Result<()> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn mouse_up(&self, _button: MouseButton, _point: Point) -> Result<()> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn scroll(&self, _horizontal: i32, _vertical: i32) -> Result<()> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn key_press(&self, _key: Key) -> Result<()> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn key_down(&self, _key: Key) -> Result<()> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn key_up(&self, _key: Key) -> Result<()> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn type_text(&self, _text: &str) -> Result<()> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn read_clipboard(&self) -> Result<Option<String>> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn write_clipboard(&self, _text: &str) -> Result<()> {
        Err(Error::UnsupportedPlatform)
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
