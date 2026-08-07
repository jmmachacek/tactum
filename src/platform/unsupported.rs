use crate::{Error, Point, Result};

pub(crate) struct Computer;

impl Computer {
    pub(crate) fn new() -> Result<Self> {
        Err(Error::UnsupportedPlatform)
    }

    pub(crate) fn click(&self, _point: Point) -> Result<()> {
        Err(Error::UnsupportedPlatform)
    }
}
