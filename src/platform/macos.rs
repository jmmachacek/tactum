use std::{thread, time::Duration};

use core_graphics::{
    access::ScreenCaptureAccess,
    display::CGDisplay,
    event::{
        CGEvent, CGEventTapLocation, CGEventType, CGKeyCode, CGMouseButton, EventField,
        ScrollEventUnit,
    },
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};
use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
use objc2_foundation::NSString;

use crate::{
    Error, Key, MouseButton, Point, Result,
    platform::{CapturedDisplay, CapturedScreenshot},
};

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> u8;
}

pub(crate) struct Computer;

const INPUT_EVENT_DELAY: Duration = Duration::from_millis(30);

struct CapturedPixels {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    desktop_origin: Point,
    desktop_width: f64,
    desktop_height: f64,
}

fn input_permission_granted() -> bool {
    // AXIsProcessTrusted has no arguments and returns the macOS Boolean type.
    unsafe { AXIsProcessTrusted() != 0 }
}

fn get_mouse_events(button: MouseButton) -> (CGMouseButton, CGEventType, CGEventType, CGEventType) {
    match button {
        MouseButton::Left => (
            CGMouseButton::Left,
            CGEventType::LeftMouseDown,
            CGEventType::LeftMouseUp,
            CGEventType::LeftMouseDragged,
        ),
        MouseButton::Right => (
            CGMouseButton::Right,
            CGEventType::RightMouseDown,
            CGEventType::RightMouseUp,
            CGEventType::RightMouseDragged,
        ),
        MouseButton::Middle => (
            CGMouseButton::Center,
            CGEventType::OtherMouseDown,
            CGEventType::OtherMouseUp,
            CGEventType::OtherMouseDragged,
        ),
    }
}

fn get_key_code(key: Key) -> CGKeyCode {
    match key {
        Key::A => 0,
        Key::B => 11,
        Key::C => 8,
        Key::D => 2,
        Key::E => 14,
        Key::F => 3,
        Key::G => 5,
        Key::H => 4,
        Key::I => 34,
        Key::J => 38,
        Key::K => 40,
        Key::L => 37,
        Key::M => 46,
        Key::N => 45,
        Key::O => 31,
        Key::P => 35,
        Key::Q => 12,
        Key::R => 15,
        Key::S => 1,
        Key::T => 17,
        Key::U => 32,
        Key::V => 9,
        Key::W => 13,
        Key::X => 7,
        Key::Y => 16,
        Key::Z => 6,
        Key::Backspace => 51,
        Key::Tab => 48,
        Key::Return => 36,
        Key::Escape => 53,
        Key::Space => 49,
        Key::Delete => 117,
        Key::Home => 115,
        Key::End => 119,
        Key::PageUp => 116,
        Key::PageDown => 121,
        Key::Left => 123,
        Key::Right => 124,
        Key::Down => 125,
        Key::Up => 126,
        Key::Shift => 56,
        Key::Control => 59,
        Key::Option => 58,
        Key::Command => 55,
    }
}

impl Computer {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self)
    }

    pub(crate) fn click(&self, button: MouseButton, point: Point) -> Result<()> {
        if !input_permission_granted() {
            return Err(Error::PermissionDenied);
        }

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| Error::OperationFailed)?;
        self.post_mouse_click(source, button, point, 1)
    }

    pub(crate) fn double_click(&self, button: MouseButton, point: Point) -> Result<()> {
        if !input_permission_granted() {
            return Err(Error::PermissionDenied);
        }

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| Error::OperationFailed)?;
        self.post_mouse_click(source.clone(), button, point, 1)?;
        thread::sleep(INPUT_EVENT_DELAY);
        self.post_mouse_click(source, button, point, 2)
    }

    pub(crate) fn mouse_down(&self, button: MouseButton, point: Point) -> Result<()> {
        let (mouse_button, down_type, _, _) = get_mouse_events(button);
        self.post_mouse_event(mouse_button, down_type, point)
    }

    pub(crate) fn mouse_up(&self, button: MouseButton, point: Point) -> Result<()> {
        let (mouse_button, _, up_type, _) = get_mouse_events(button);
        self.post_mouse_event(mouse_button, up_type, point)
    }

    fn post_mouse_click(
        &self,
        source: CGEventSource,
        button: MouseButton,
        point: Point,
        click_state: i64,
    ) -> Result<()> {
        let (mouse_button, down_type, up_type, _) = get_mouse_events(button);
        let location = CGPoint::new(point.x(), point.y());
        let down = CGEvent::new_mouse_event(source.clone(), down_type, location, mouse_button)
            .map_err(|_| Error::OperationFailed)?;
        let up = CGEvent::new_mouse_event(source, up_type, location, mouse_button)
            .map_err(|_| Error::OperationFailed)?;

        down.set_integer_value_field(EventField::MOUSE_EVENT_CLICK_STATE, click_state);
        up.set_integer_value_field(EventField::MOUSE_EVENT_CLICK_STATE, click_state);
        down.post(CGEventTapLocation::HID);
        up.post(CGEventTapLocation::HID);
        Ok(())
    }

    fn post_mouse_event(
        &self,
        button: CGMouseButton,
        event_type: CGEventType,
        point: Point,
    ) -> Result<()> {
        if !input_permission_granted() {
            return Err(Error::PermissionDenied);
        }

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| Error::OperationFailed)?;
        let event = CGEvent::new_mouse_event(
            source,
            event_type,
            CGPoint::new(point.x(), point.y()),
            button,
        )
        .map_err(|_| Error::OperationFailed)?;
        event.post(CGEventTapLocation::HID);
        Ok(())
    }

    pub(crate) fn move_to(&self, point: Point) -> Result<()> {
        if !input_permission_granted() {
            return Err(Error::PermissionDenied);
        }

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| Error::OperationFailed)?;
        let event = CGEvent::new_mouse_event(
            source,
            CGEventType::MouseMoved,
            CGPoint::new(point.x(), point.y()),
            CGMouseButton::Left,
        )
        .map_err(|_| Error::OperationFailed)?;

        event.post(CGEventTapLocation::HID);
        Ok(())
    }

    pub(crate) fn drag(&self, button: MouseButton, from: Point, to: Point) -> Result<()> {
        if !input_permission_granted() {
            return Err(Error::PermissionDenied);
        }

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| Error::OperationFailed)?;
        let (mouse_button, down_type, up_type, dragged_type) = get_mouse_events(button);
        let down = CGEvent::new_mouse_event(
            source.clone(),
            down_type,
            CGPoint::new(from.x(), from.y()),
            mouse_button,
        )
        .map_err(|_| Error::OperationFailed)?;
        let dragged = CGEvent::new_mouse_event(
            source.clone(),
            dragged_type,
            CGPoint::new(to.x(), to.y()),
            mouse_button,
        )
        .map_err(|_| Error::OperationFailed)?;
        let up =
            CGEvent::new_mouse_event(source, up_type, CGPoint::new(to.x(), to.y()), mouse_button)
                .map_err(|_| Error::OperationFailed)?;

        down.post(CGEventTapLocation::HID);
        thread::sleep(INPUT_EVENT_DELAY);
        dragged.post(CGEventTapLocation::HID);
        thread::sleep(INPUT_EVENT_DELAY);
        up.post(CGEventTapLocation::HID);
        Ok(())
    }

    pub(crate) fn scroll(&self, horizontal: i32, vertical: i32) -> Result<()> {
        if horizontal == 0 && vertical == 0 {
            return Ok(());
        }
        if !input_permission_granted() {
            return Err(Error::PermissionDenied);
        }

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| Error::OperationFailed)?;
        let event =
            CGEvent::new_scroll_event(source, ScrollEventUnit::LINE, 2, vertical, horizontal, 0)
                .map_err(|_| Error::OperationFailed)?;
        event.post(CGEventTapLocation::HID);
        Ok(())
    }

    pub(crate) fn key_press(&self, key: Key) -> Result<()> {
        self.key_down(key)?;
        self.key_up(key)
    }

    pub(crate) fn key_down(&self, key: Key) -> Result<()> {
        self.post_key(get_key_code(key), true)
    }

    pub(crate) fn key_up(&self, key: Key) -> Result<()> {
        self.post_key(get_key_code(key), false)
    }

    pub(crate) fn type_text(&self, text: &str) -> Result<()> {
        if !input_permission_granted() {
            return Err(Error::PermissionDenied);
        }

        for character in text.chars() {
            match character {
                '\n' | '\r' => self.key_press(Key::Return)?,
                '\t' => self.key_press(Key::Tab)?,
                _ => {
                    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
                        .map_err(|_| Error::OperationFailed)?;
                    let down = CGEvent::new_keyboard_event(source.clone(), 0, true)
                        .map_err(|_| Error::OperationFailed)?;
                    let up = CGEvent::new_keyboard_event(source, 0, false)
                        .map_err(|_| Error::OperationFailed)?;
                    let text = character.to_string();

                    down.set_string(&text);
                    up.set_string(&text);
                    down.post(CGEventTapLocation::HID);
                    up.post(CGEventTapLocation::HID);
                }
            }
        }

        Ok(())
    }

    pub(crate) fn read_clipboard(&self) -> Result<Option<String>> {
        let pasteboard = NSPasteboard::generalPasteboard();
        let string_type = unsafe { NSPasteboardTypeString };

        Ok(pasteboard
            .stringForType(string_type)
            .map(|text| text.to_string()))
    }

    pub(crate) fn write_clipboard(&self, text: &str) -> Result<()> {
        let pasteboard = NSPasteboard::generalPasteboard();
        let string_type = unsafe { NSPasteboardTypeString };
        pasteboard.clearContents();

        if pasteboard.setString_forType(&NSString::from_str(text), string_type) {
            Ok(())
        } else {
            Err(Error::OperationFailed)
        }
    }

    fn post_key(&self, key_code: CGKeyCode, key_down: bool) -> Result<()> {
        if !input_permission_granted() {
            return Err(Error::PermissionDenied);
        }

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| Error::OperationFailed)?;
        let event = CGEvent::new_keyboard_event(source, key_code, key_down)
            .map_err(|_| Error::OperationFailed)?;
        event.post(CGEventTapLocation::HID);
        Ok(())
    }

    pub(crate) fn displays(&self) -> Result<Vec<CapturedDisplay>> {
        CGDisplay::active_displays()
            .map_err(|_| Error::OperationFailed)?
            .into_iter()
            .map(|id| {
                let display = CGDisplay::new(id);
                let bounds = display.bounds();
                let width = bounds.size.width;
                let height = bounds.size.height;
                if width <= 0.0 || height <= 0.0 {
                    return Err(Error::OperationFailed);
                }
                let mode = display.display_mode().ok_or(Error::OperationFailed)?;

                Ok(CapturedDisplay {
                    id: u64::from(id),
                    origin: Point::new(bounds.origin.x, bounds.origin.y)
                        .map_err(|_| Error::OperationFailed)?,
                    width,
                    height,
                    scale_x: mode.pixel_width() as f64 / width,
                    scale_y: mode.pixel_height() as f64 / height,
                })
            })
            .collect()
    }

    pub(crate) fn screenshot(&self) -> Result<CapturedScreenshot> {
        self.capture_screenshot(CGDisplay::main())
    }

    pub(crate) fn screenshot_display(&self, id: u64) -> Result<CapturedScreenshot> {
        let id = u32::try_from(id).map_err(|_| Error::OperationFailed)?;
        self.capture_screenshot(CGDisplay::new(id))
    }

    pub(crate) fn screenshot_all_displays(&self) -> Result<CapturedScreenshot> {
        if !ScreenCaptureAccess.preflight() {
            return Err(Error::PermissionDenied);
        }

        let captures: Vec<_> = CGDisplay::active_displays()
            .map_err(|_| Error::OperationFailed)?
            .into_iter()
            .map(|id| self.capture_pixels(CGDisplay::new(id)))
            .collect::<Result<_>>()?;
        let first = captures.first().ok_or(Error::OperationFailed)?;
        let (min_x, min_y, max_x, max_y, scale) = captures.iter().skip(1).fold(
            (
                first.desktop_origin.x(),
                first.desktop_origin.y(),
                first.desktop_origin.x() + first.desktop_width,
                first.desktop_origin.y() + first.desktop_height,
                (first.width as f64 / first.desktop_width)
                    .max(first.height as f64 / first.desktop_height),
            ),
            |(min_x, min_y, max_x, max_y, scale), capture| {
                (
                    min_x.min(capture.desktop_origin.x()),
                    min_y.min(capture.desktop_origin.y()),
                    max_x.max(capture.desktop_origin.x() + capture.desktop_width),
                    max_y.max(capture.desktop_origin.y() + capture.desktop_height),
                    scale
                        .max(capture.width as f64 / capture.desktop_width)
                        .max(capture.height as f64 / capture.desktop_height),
                )
            },
        );
        let desktop_width = max_x - min_x;
        let desktop_height = max_y - min_y;
        let width = pixels_for_desktop_length(desktop_width, scale)?;
        let height = pixels_for_desktop_length(desktop_height, scale)?;
        let pixel_count = usize::try_from(width)
            .ok()
            .and_then(|width| width.checked_mul(height as usize))
            .ok_or(Error::OperationFailed)?;
        let mut rgba = vec![[0, 0, 0, u8::MAX]; pixel_count].into_flattened();

        for capture in &captures {
            let x = pixels_for_desktop_length(capture.desktop_origin.x() - min_x, scale)?;
            let y = pixels_for_desktop_length(capture.desktop_origin.y() - min_y, scale)?;
            let right = pixels_for_desktop_length(
                capture.desktop_origin.x() + capture.desktop_width - min_x,
                scale,
            )?;
            let bottom = pixels_for_desktop_length(
                capture.desktop_origin.y() + capture.desktop_height - min_y,
                scale,
            )?;
            let capture_width = right.checked_sub(x).ok_or(Error::OperationFailed)?;
            let capture_height = bottom.checked_sub(y).ok_or(Error::OperationFailed)?;
            if capture_width == 0 || capture_height == 0 || right > width || bottom > height {
                return Err(Error::OperationFailed);
            }

            for target_y in 0..capture_height {
                let source_y =
                    target_y as usize * capture.height as usize / capture_height as usize;
                for target_x in 0..capture_width {
                    let source_x =
                        target_x as usize * capture.width as usize / capture_width as usize;
                    let source = (source_y * capture.width as usize + source_x) * 4;
                    let target = ((y as usize + target_y as usize) * width as usize
                        + x as usize
                        + target_x as usize)
                        * 4;
                    rgba[target..target + 4].copy_from_slice(&capture.rgba[source..source + 4]);
                }
            }
        }

        encode_screenshot(
            rgba,
            width,
            height,
            Point::new(min_x, min_y).map_err(|_| Error::OperationFailed)?,
            desktop_width,
            desktop_height,
        )
    }

    fn capture_screenshot(&self, display: CGDisplay) -> Result<CapturedScreenshot> {
        if !ScreenCaptureAccess.preflight() {
            return Err(Error::PermissionDenied);
        }

        let capture = self.capture_pixels(display)?;
        encode_screenshot(
            capture.rgba,
            capture.width,
            capture.height,
            capture.desktop_origin,
            capture.desktop_width,
            capture.desktop_height,
        )
    }

    fn capture_pixels(&self, display: CGDisplay) -> Result<CapturedPixels> {
        let display_bounds = display.bounds();
        let image = display.image().ok_or(Error::OperationFailed)?;
        let width = u32::try_from(image.width()).map_err(|_| Error::OperationFailed)?;
        let height = u32::try_from(image.height()).map_err(|_| Error::OperationFailed)?;
        let row_length = usize::try_from(width)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or(Error::OperationFailed)?;
        let bytes_per_row = image.bytes_per_row();
        let data = image.data();
        let data = data.bytes();
        if image.bits_per_component() != 8
            || image.bits_per_pixel() != 32
            || bytes_per_row < row_length
            || data.len()
                < bytes_per_row
                    .checked_mul(height as usize)
                    .ok_or(Error::OperationFailed)?
        {
            return Err(Error::OperationFailed);
        }

        // CGDisplay images are 32-bit little-endian BGRA. Pack rows and convert to RGBA.
        let mut rgba = Vec::with_capacity(row_length * height as usize);
        for row in data.chunks_exact(bytes_per_row).take(height as usize) {
            rgba.extend_from_slice(&row[..row_length]);
        }
        for pixel in rgba.as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
        }

        let desktop_origin = Point::new(display_bounds.origin.x, display_bounds.origin.y)
            .map_err(|_| Error::OperationFailed)?;
        Ok(CapturedPixels {
            rgba,
            width,
            height,
            desktop_origin,
            desktop_width: display_bounds.size.width,
            desktop_height: display_bounds.size.height,
        })
    }
}

fn pixels_for_desktop_length(length: f64, scale: f64) -> Result<u32> {
    if !length.is_finite() || !scale.is_finite() || length < 0.0 || scale <= 0.0 {
        return Err(Error::OperationFailed);
    }

    let pixels = (length * scale).round();
    if pixels > u32::MAX as f64 {
        return Err(Error::OperationFailed);
    }

    Ok(pixels as u32)
}

fn encode_screenshot(
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    desktop_origin: Point,
    desktop_width: f64,
    desktop_height: f64,
) -> Result<CapturedScreenshot> {
    let mut png = Vec::new();
    PngEncoder::new(&mut png)
        .write_image(&rgba, width, height, ExtendedColorType::Rgba8)
        .map_err(|_| Error::OperationFailed)?;

    Ok(CapturedScreenshot {
        png,
        width,
        height,
        desktop_origin,
        desktop_width,
        desktop_height,
    })
}
