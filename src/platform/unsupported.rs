use crate::{Error, Point, Result, Screenshot};

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

    pub(crate) fn screenshot(&self) -> Result<Screenshot> {
        Err(Error::UnsupportedPlatform)
    }
}
