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

/// A mouse button.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// A keyboard key represented by its physical position on a standard US layout.
///
/// Letter variants follow US QWERTY positions and may produce different
/// characters with another active keyboard layout. Use [`Computer::type_text`]
/// to enter layout-independent text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Backspace,
    Tab,
    Return,
    Escape,
    Space,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Left,
    Right,
    Down,
    Up,
    Shift,
    Control,
    Option,
    Command,
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

    /// Clicks `button` at `point`.
    pub fn click(&self, button: MouseButton, point: Point) -> Result<()> {
        self.platform.click(button, point)
    }

    /// Double-clicks `button` at `point`.
    pub fn double_click(&self, button: MouseButton, point: Point) -> Result<()> {
        self.platform.double_click(button, point)
    }

    /// Moves the mouse cursor to `point`.
    pub fn move_to(&self, point: Point) -> Result<()> {
        self.platform.move_to(point)
    }

    /// Drags `button` from `from` to `to`.
    pub fn drag(&self, button: MouseButton, from: Point, to: Point) -> Result<()> {
        self.platform.drag(button, from, to)
    }

    /// Scrolls the focused target by signed logical deltas.
    ///
    /// Positive horizontal values scroll right; positive vertical values scroll up.
    /// The actual distance is determined by the operating system and target application.
    pub fn scroll(&self, horizontal: i32, vertical: i32) -> Result<()> {
        self.platform.scroll(horizontal, vertical)
    }

    /// Presses and releases `key`.
    pub fn key_press(&self, key: Key) -> Result<()> {
        self.platform.key_press(key)
    }

    /// Presses `key` until [`Computer::key_up`] is called.
    pub fn key_down(&self, key: Key) -> Result<()> {
        self.platform.key_down(key)
    }

    /// Releases `key`.
    pub fn key_up(&self, key: Key) -> Result<()> {
        self.platform.key_up(key)
    }

    /// Types Unicode text into the focused application.
    pub fn type_text(&self, text: &str) -> Result<()> {
        self.platform.type_text(text)
    }

    /// Returns plain text from the system clipboard, if available.
    pub fn read_clipboard(&self) -> Result<Option<String>> {
        self.platform.read_clipboard()
    }

    /// Replaces the system clipboard with plain text.
    pub fn write_clipboard(&self, text: &str) -> Result<()> {
        self.platform.write_clipboard(text)
    }

    /// Captures the primary display as a PNG image.
    pub fn screenshot(&self) -> Result<Screenshot> {
        let captured = self.platform.screenshot()?;

        Ok(Screenshot {
            png: captured.png,
            width: captured.width,
            height: captured.height,
            desktop_origin: captured.desktop_origin,
            desktop_width: captured.desktop_width,
            desktop_height: captured.desktop_height,
        })
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
        let screenshot = Screenshot {
            png: Vec::new(),
            width: 200,
            height: 100,
            desktop_origin: Point::new(-100.0, 50.0).unwrap(),
            desktop_width: 100.0,
            desktop_height: 50.0,
        };

        let point = screenshot.to_desktop_point(100.0, 50.0).unwrap();

        assert_eq!(point.x(), -50.0);
        assert_eq!(point.y(), 75.0);
        assert_eq!(
            screenshot.to_desktop_point(200.0, 0.0),
            Err(Error::InvalidPoint)
        );
    }
}
