use crate::Point;

pub(crate) struct CapturedScreenshot {
    pub(crate) png: Vec<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) desktop_origin: Point,
    pub(crate) desktop_width: f64,
    pub(crate) desktop_height: f64,
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(crate) use macos::Computer;

#[cfg(not(target_os = "macos"))]
mod unsupported;
#[cfg(not(target_os = "macos"))]
pub(crate) use unsupported::Computer;
