use crate::{Error, Key, Point, Result, platform::CapturedScreenshot};

pub(crate) struct Computer;

impl Computer {
    pub(crate) fn new() -> Result<Self> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn click(&self, _point: Point) -> Result<()> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn move_to(&self, _point: Point) -> Result<()> {
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

    pub(crate) fn screenshot(&self) -> Result<CapturedScreenshot> {
        Err(Error::UnsupportedPlatform)
    }
}
