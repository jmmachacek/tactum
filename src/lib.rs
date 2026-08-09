//! Cross-platform computer control primitives.

mod platform;

use std::fmt;

/// A point in the desktop's global coordinate space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    /// Creates a point with finite coordinates.
    pub fn new(x: f64, y: f64) -> Result<Self> {
        if !x.is_finite() || !y.is_finite() {
            return Err(Error::InvalidPoint);
        }

        Ok(Self { x, y })
    }

    /// Returns the horizontal coordinate.
    pub const fn x(self) -> f64 {
        self.x
    }

    /// Returns the vertical coordinate.
    pub const fn y(self) -> f64 {
        self.y
    }
}

/// A controller for the local computer.
pub struct Computer {
    platform: platform::Computer,
}

impl Computer {
    /// Connects to the local computer-control backend.
    pub fn new() -> Result<Self> {
        Ok(Self {
            platform: platform::Computer::new()?,
        })
    }

    /// Clicks the primary mouse button at `point`.
    pub fn click(&self, point: Point) -> Result<()> {
        self.platform.click(point)
    }

    /// Captures the primary display as a PNG image.
    pub fn screenshot(&self) -> Result<Screenshot> {
        self.platform.screenshot()
    }
}

/// A PNG screenshot of a display.
#[derive(Clone, Debug, PartialEq)]
pub struct Screenshot {
    png: Vec<u8>,
    width: u32,
    height: u32,
    desktop_origin: Point,
    desktop_width: f64,
    desktop_height: f64,
}

impl Screenshot {
    pub(crate) fn new(
        png: Vec<u8>,
        width: u32,
        height: u32,
        desktop_origin: Point,
        desktop_width: f64,
        desktop_height: f64,
    ) -> Self {
        Self {
            png,
            width,
            height,
            desktop_origin,
            desktop_width,
            desktop_height,
        }
    }

    /// Returns the screenshot width in pixels.
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the screenshot height in pixels.
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the PNG-encoded image data.
    pub fn png(&self) -> &[u8] {
        &self.png
    }

    /// Consumes the screenshot and returns its PNG-encoded image data.
    pub fn into_png(self) -> Vec<u8> {
        self.png
    }

    /// Converts a pixel coordinate in this image to a global desktop point.
    pub fn to_desktop_point(&self, x: f64, y: f64) -> Result<Point> {
        if !x.is_finite()
            || !y.is_finite()
            || x < 0.0
            || y < 0.0
            || x >= self.width as f64
            || y >= self.height as f64
        {
            return Err(Error::InvalidPoint);
        }

        Point::new(
            self.desktop_origin.x() + x * self.desktop_width / self.width as f64,
            self.desktop_origin.y() + y * self.desktop_height / self.height as f64,
        )
    }
}

/// Errors returned by computer-control operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The current operating system does not have a backend yet.
    UnsupportedPlatform,
    /// A coordinate was invalid for the requested operation.
    InvalidPoint,
    /// The process is not allowed to perform the requested operation.
    PermissionDenied,
    /// The requested computer-control operation failed.
    OperationFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => f.write_str("this platform is not supported yet"),
            Self::InvalidPoint => f.write_str("coordinates are invalid for this operation"),
            Self::PermissionDenied => {
                f.write_str("operating system permission is required for this operation")
            }
            Self::OperationFailed => f.write_str("the computer-control operation failed"),
        }
    }
}

impl std::error::Error for Error {}

/// A specialized result type for Tactum operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::{Error, Point, Screenshot};

    #[test]
    fn point_accepts_finite_coordinates() {
        let point = Point::new(-12.5, 0.0).unwrap();

        assert_eq!(point.x(), -12.5);
        assert_eq!(point.y(), 0.0);
    }

    #[test]
    fn point_rejects_invalid_coordinates() {
        assert_eq!(Point::new(f64::NAN, 0.0), Err(Error::InvalidPoint));
        assert_eq!(Point::new(0.0, f64::INFINITY), Err(Error::InvalidPoint));
    }

    #[test]
    fn screenshot_maps_pixels_to_desktop_points() {
        let screenshot = Screenshot::new(
            Vec::new(),
            200,
            100,
            Point::new(-100.0, 50.0).unwrap(),
            100.0,
            50.0,
        );

        let point = screenshot.to_desktop_point(100.0, 50.0).unwrap();

        assert_eq!(point.x(), -50.0);
        assert_eq!(point.y(), 75.0);
        assert_eq!(
            screenshot.to_desktop_point(200.0, 0.0),
            Err(Error::InvalidPoint)
        );
    }
}
