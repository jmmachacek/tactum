# Tactum

Tactum is a Rust library that gives AI agents a small, platform-independent API
for controlling the local computer on macOS and Windows. It provides the core
primitives needed to observe and interact with a graphical desktop.

Agents can capture the screen, map image coordinates back to the desktop, and
interact through mouse, keyboard, and clipboard operations.

## Features

- Move, click, drag, and scroll the mouse
- Press keys and type Unicode text
- Read and write plain text on the clipboard
- Discover connected displays
- Capture individual displays or the entire desktop as PNG images
- Convert screenshot pixel coordinates to desktop coordinates

## Example

```rust
use tactum::{Computer, MouseButton, Point};

fn main() -> tactum::Result<()> {
    let computer = Computer::new()?;
    let point = Point::new(400.0, 300.0)?;

    computer.click(MouseButton::Left, point)?;
    computer.type_text("Hello from Tactum!")?;

    Ok(())
}
```

## Platform support

Tactum currently supports macOS and Windows. Other platforms return
`Error::UnsupportedPlatform`.

On macOS, input control and screenshots may require Accessibility and Screen
Recording permissions in System Settings.
