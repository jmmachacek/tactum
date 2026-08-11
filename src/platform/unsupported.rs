use crate::{Error, Key, MouseButton, Point, Result, platform::CapturedScreenshot};

pub(crate) struct Computer;

impl Computer {
    pub(crate) fn new() -> Result<Self> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn click(&self, _button: MouseButton, _point: Point) -> Result<()> {
        Err(Error::UnsupportedPlatform)
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

    pub(crate) fn screenshot(&self) -> Result<CapturedScreenshot> {
        Err(Error::UnsupportedPlatform)
    }
}
