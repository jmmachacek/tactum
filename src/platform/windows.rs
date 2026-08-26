use std::mem::size_of;

use windows_sys::Win32::{
    Foundation::{BOOL, LPARAM, RECT},
    Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO},
    UI::{
        Input::KeyboardAndMouse::{
            INPUT, INPUT_0, INPUT_MOUSE, MOUSE_EVENT_FLAGS, MOUSEEVENTF_LEFTDOWN,
            MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
            MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT, SendInput,
        },
        WindowsAndMessaging::SetCursorPos,
    },
};

use crate::{
    Error, Key, MouseButton, Point, Result,
    platform::{CapturedDisplay, CapturedScreenshot},
};

pub(crate) struct Computer;

struct DisplayCollector {
    displays: Vec<CapturedDisplay>,
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
    1
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
        let mut collector = DisplayCollector {
            displays: Vec::new(),
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

        Ok(collector.displays)
    }

    pub(crate) fn screenshot(&self) -> Result<CapturedScreenshot> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn screenshot_display(&self, _id: u64) -> Result<CapturedScreenshot> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn screenshot_all_displays(&self) -> Result<CapturedScreenshot> {
        Err(Error::UnsupportedPlatform)
    }
}
