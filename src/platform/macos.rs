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
use objc2::rc::autoreleasepool;
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

fn input_event_source() -> Result<CGEventSource> {
    if !input_permission_granted() {
        return Err(Error::PermissionDenied);
    }

    CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| Error::OperationFailed)
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
        let source = input_event_source()?;
        post_mouse_click(create_mouse_click(source, button, point, 1)?);
        Ok(())
    }

    pub(crate) fn double_click(&self, button: MouseButton, point: Point) -> Result<()> {
        let source = input_event_source()?;
        let first = create_mouse_click(source.clone(), button, point, 1)?;
        let second = create_mouse_click(source, button, point, 2)?;

        post_mouse_click(first);
        thread::sleep(INPUT_EVENT_DELAY);
        post_mouse_click(second);
        Ok(())
    }

    pub(crate) fn mouse_down(&self, button: MouseButton, point: Point) -> Result<()> {
        let (mouse_button, down_type, _, _) = get_mouse_events(button);
        self.post_mouse_event(mouse_button, down_type, point)
    }

    pub(crate) fn mouse_up(&self, button: MouseButton, point: Point) -> Result<()> {
        let (mouse_button, _, up_type, _) = get_mouse_events(button);
        self.post_mouse_event(mouse_button, up_type, point)
    }

    fn post_mouse_event(
        &self,
        button: CGMouseButton,
        event_type: CGEventType,
        point: Point,
    ) -> Result<()> {
        let source = input_event_source()?;
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
        self.post_mouse_event(CGMouseButton::Left, CGEventType::MouseMoved, point)
    }

    pub(crate) fn drag(&self, button: MouseButton, from: Point, to: Point) -> Result<()> {
        let source = input_event_source()?;
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
        let source = input_event_source()?;
        if horizontal != 0 && vertical != 0 {
            let vertical_event = create_scroll_event(source.clone(), 0, vertical)?;
            let horizontal_event = create_scroll_event(source, horizontal, 0)?;
            vertical_event.post(CGEventTapLocation::HID);
            horizontal_event.post(CGEventTapLocation::HID);
        } else {
            create_scroll_event(source, horizontal, vertical)?.post(CGEventTapLocation::HID);
        }
        Ok(())
    }

    pub(crate) fn key_press(&self, key: Key) -> Result<()> {
        let source = input_event_source()?;
        post_key_press(source, get_key_code(key))
    }

    pub(crate) fn key_down(&self, key: Key) -> Result<()> {
        self.post_key(get_key_code(key), true)
    }

    pub(crate) fn key_up(&self, key: Key) -> Result<()> {
        self.post_key(get_key_code(key), false)
    }

    pub(crate) fn type_text(&self, text: &str) -> Result<()> {
        let source = input_event_source()?;

        for character in text.chars() {
            match character {
                '\n' | '\r' => post_key_press(source.clone(), get_key_code(Key::Return))?,
                '\t' => post_key_press(source.clone(), get_key_code(Key::Tab))?,
                _ => {
                    let down = CGEvent::new_keyboard_event(source.clone(), 0, true)
                        .map_err(|_| Error::OperationFailed)?;
                    let up = CGEvent::new_keyboard_event(source.clone(), 0, false)
                        .map_err(|_| Error::OperationFailed)?;
                    let mut utf16 = [0; 2];
                    let utf16 = character.encode_utf16(&mut utf16);

                    down.set_string_from_utf16_unchecked(utf16);
                    up.set_string_from_utf16_unchecked(utf16);
                    down.post(CGEventTapLocation::HID);
                    up.post(CGEventTapLocation::HID);
                }
            }
        }

        Ok(())
    }

    pub(crate) fn read_clipboard(&self) -> Result<Option<String>> {
        Ok(autoreleasepool(|_| {
            let pasteboard = NSPasteboard::generalPasteboard();
            let string_type = unsafe { NSPasteboardTypeString };

            pasteboard
                .stringForType(string_type)
                .map(|text| text.to_string())
        }))
    }

    pub(crate) fn write_clipboard(&self, text: &str) -> Result<()> {
        let written = autoreleasepool(|_| {
            let pasteboard = NSPasteboard::generalPasteboard();
            let string_type = unsafe { NSPasteboardTypeString };
            pasteboard.clearContents();
            pasteboard.setString_forType(&NSString::from_str(text), string_type)
        });

        if written {
            Ok(())
        } else {
            Err(Error::OperationFailed)
        }
    }

    fn post_key(&self, key_code: CGKeyCode, key_down: bool) -> Result<()> {
        let source = input_event_source()?;
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
                if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
                    return Err(Error::OperationFailed);
                }
                let mode = display.display_mode().ok_or(Error::OperationFailed)?;
                let pixel_width = mode.pixel_width();
                let pixel_height = mode.pixel_height();
                if pixel_width == 0 || pixel_height == 0 {
                    return Err(Error::OperationFailed);
                }
                let scale_x = pixel_width as f64 / width;
                let scale_y = pixel_height as f64 / height;
                if !scale_x.is_finite() || !scale_y.is_finite() {
                    return Err(Error::OperationFailed);
                }

                Ok(CapturedDisplay {
                    id: u64::from(id),
                    origin: Point::new(bounds.origin.x, bounds.origin.y)
                        .map_err(|_| Error::OperationFailed)?,
                    width,
                    height,
                    scale_x,
                    scale_y,
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

        let captures = CGDisplay::active_displays()
            .map_err(|_| Error::OperationFailed)?
            .into_iter()
            .map(|id| self.capture_pixels(CGDisplay::new(id)))
            .collect::<Result<_>>()?;
        let capture = compose_captures(captures)?;
        encode_screenshot(capture)
    }

    fn capture_screenshot(&self, display: CGDisplay) -> Result<CapturedScreenshot> {
        if !ScreenCaptureAccess.preflight() {
            return Err(Error::PermissionDenied);
        }

        let capture = self.capture_pixels(display)?;
        encode_screenshot(capture)
    }

    fn capture_pixels(&self, display: CGDisplay) -> Result<CapturedPixels> {
        let display_bounds = display.bounds();
        let desktop_width = display_bounds.size.width;
        let desktop_height = display_bounds.size.height;
        if !desktop_width.is_finite()
            || !desktop_height.is_finite()
            || desktop_width <= 0.0
            || desktop_height <= 0.0
        {
            return Err(Error::OperationFailed);
        }
        let image = display.image().ok_or(Error::OperationFailed)?;
        let width = u32::try_from(image.width()).map_err(|_| Error::OperationFailed)?;
        let height = u32::try_from(image.height()).map_err(|_| Error::OperationFailed)?;
        if width == 0 || height == 0 {
            return Err(Error::OperationFailed);
        }
        let row_length = usize::try_from(width)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or(Error::OperationFailed)?;
        let rgba_length = row_length
            .checked_mul(height as usize)
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
        let mut rgba = Vec::with_capacity(rgba_length);
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
            desktop_width,
            desktop_height,
        })
    }
}

fn create_mouse_click(
    source: CGEventSource,
    button: MouseButton,
    point: Point,
    click_state: i64,
) -> Result<(CGEvent, CGEvent)> {
    let (mouse_button, down_type, up_type, _) = get_mouse_events(button);
    let location = CGPoint::new(point.x(), point.y());
    let down = CGEvent::new_mouse_event(source.clone(), down_type, location, mouse_button)
        .map_err(|_| Error::OperationFailed)?;
    let up = CGEvent::new_mouse_event(source, up_type, location, mouse_button)
        .map_err(|_| Error::OperationFailed)?;

    down.set_integer_value_field(EventField::MOUSE_EVENT_CLICK_STATE, click_state);
    up.set_integer_value_field(EventField::MOUSE_EVENT_CLICK_STATE, click_state);
    Ok((down, up))
}

fn post_mouse_click((down, up): (CGEvent, CGEvent)) {
    down.post(CGEventTapLocation::HID);
    up.post(CGEventTapLocation::HID);
}

fn post_key_press(source: CGEventSource, key_code: CGKeyCode) -> Result<()> {
    let down = CGEvent::new_keyboard_event(source.clone(), key_code, true)
        .map_err(|_| Error::OperationFailed)?;
    let up =
        CGEvent::new_keyboard_event(source, key_code, false).map_err(|_| Error::OperationFailed)?;

    down.post(CGEventTapLocation::HID);
    up.post(CGEventTapLocation::HID);
    Ok(())
}

fn create_scroll_event(source: CGEventSource, horizontal: i32, vertical: i32) -> Result<CGEvent> {
    let wheel_count = if horizontal == 0 { 1 } else { 2 };
    CGEvent::new_scroll_event(
        source,
        ScrollEventUnit::LINE,
        wheel_count,
        vertical,
        horizontal,
        0,
    )
    .map_err(|_| Error::OperationFailed)
}

fn compose_captures(mut captures: Vec<CapturedPixels>) -> Result<CapturedPixels> {
    if captures.is_empty() {
        return Err(Error::OperationFailed);
    }
    for capture in &captures {
        validate_capture(capture)?;
    }
    if captures.len() == 1 {
        return captures.pop().ok_or(Error::OperationFailed);
    }

    let first = captures.first().ok_or(Error::OperationFailed)?;
    let mut min_x = first.desktop_origin.x();
    let mut min_y = first.desktop_origin.y();
    let mut max_x = min_x + first.desktop_width;
    let mut max_y = min_y + first.desktop_height;
    let mut scale = capture_scale(first);

    for capture in captures.iter().skip(1) {
        min_x = min_x.min(capture.desktop_origin.x());
        min_y = min_y.min(capture.desktop_origin.y());
        max_x = max_x.max(capture.desktop_origin.x() + capture.desktop_width);
        max_y = max_y.max(capture.desktop_origin.y() + capture.desktop_height);
        scale = scale.max(capture_scale(capture));
    }

    let desktop_width = max_x - min_x;
    let desktop_height = max_y - min_y;
    let width = pixels_for_desktop_length(desktop_width, scale)?;
    let height = pixels_for_desktop_length(desktop_height, scale)?;
    let pixel_count = (width as usize)
        .checked_mul(height as usize)
        .ok_or(Error::OperationFailed)?;
    pixel_count.checked_mul(4).ok_or(Error::OperationFailed)?;
    let mut rgba = vec![[0, 0, 0, u8::MAX]; pixel_count].into_flattened();

    for capture in &captures {
        copy_capture(&mut rgba, width, height, min_x, min_y, scale, capture)?;
    }

    Ok(CapturedPixels {
        rgba,
        width,
        height,
        desktop_origin: Point::new(min_x, min_y).map_err(|_| Error::OperationFailed)?,
        desktop_width,
        desktop_height,
    })
}

fn validate_capture(capture: &CapturedPixels) -> Result<()> {
    let expected_length = (capture.width as usize)
        .checked_mul(capture.height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(Error::OperationFailed)?;
    if capture.width == 0
        || capture.height == 0
        || capture.rgba.len() != expected_length
        || !capture.desktop_width.is_finite()
        || !capture.desktop_height.is_finite()
        || capture.desktop_width <= 0.0
        || capture.desktop_height <= 0.0
    {
        return Err(Error::OperationFailed);
    }
    Ok(())
}

fn capture_scale(capture: &CapturedPixels) -> f64 {
    (capture.width as f64 / capture.desktop_width)
        .max(capture.height as f64 / capture.desktop_height)
}

fn copy_capture(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    min_x: f64,
    min_y: f64,
    scale: f64,
    capture: &CapturedPixels,
) -> Result<()> {
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
    let target_width = right.checked_sub(x).ok_or(Error::OperationFailed)?;
    let target_height = bottom.checked_sub(y).ok_or(Error::OperationFailed)?;
    if target_width == 0 || target_height == 0 || right > width || bottom > height {
        return Err(Error::OperationFailed);
    }

    if target_width == capture.width && target_height == capture.height {
        let row_length = capture.width as usize * 4;
        for (source_y, row) in capture.rgba.chunks_exact(row_length).enumerate() {
            let target = ((y as usize + source_y) * width as usize + x as usize) * 4;
            rgba[target..target + row_length].copy_from_slice(row);
        }
        return Ok(());
    }

    for target_y in 0..target_height {
        let source_y = target_y as usize * capture.height as usize / target_height as usize;
        for target_x in 0..target_width {
            let source_x = target_x as usize * capture.width as usize / target_width as usize;
            let source = (source_y * capture.width as usize + source_x) * 4;
            let target = ((y as usize + target_y as usize) * width as usize
                + x as usize
                + target_x as usize)
                * 4;
            rgba[target..target + 4].copy_from_slice(&capture.rgba[source..source + 4]);
        }
    }
    Ok(())
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

fn encode_screenshot(capture: CapturedPixels) -> Result<CapturedScreenshot> {
    let CapturedPixels {
        rgba,
        width,
        height,
        desktop_origin,
        desktop_width,
        desktop_height,
    } = capture;
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

#[cfg(test)]
mod tests {
    use super::{CapturedPixels, compose_captures};
    use crate::{Error, Point};

    #[test]
    fn composing_one_capture_reuses_its_pixels() {
        let rgba = vec![1, 2, 3, 4];
        let allocation = rgba.as_ptr();

        let composed = compose_captures(vec![CapturedPixels {
            rgba,
            width: 1,
            height: 1,
            desktop_origin: Point::new(5.0, 6.0).unwrap(),
            desktop_width: 1.0,
            desktop_height: 1.0,
        }])
        .unwrap();

        assert_eq!(composed.rgba.as_ptr(), allocation);
        assert_eq!(composed.desktop_origin, Point::new(5.0, 6.0).unwrap());
    }

    #[test]
    fn composing_captures_preserves_black_desktop_gaps() {
        let left = CapturedPixels {
            rgba: vec![255, 0, 0, 255],
            width: 1,
            height: 1,
            desktop_origin: Point::new(-1.0, 0.0).unwrap(),
            desktop_width: 1.0,
            desktop_height: 1.0,
        };
        let right = CapturedPixels {
            rgba: vec![0, 0, 255, 255],
            width: 1,
            height: 1,
            desktop_origin: Point::new(1.0, 0.0).unwrap(),
            desktop_width: 1.0,
            desktop_height: 1.0,
        };

        let composed = compose_captures(vec![left, right]).unwrap();

        assert_eq!(composed.width, 3);
        assert_eq!(composed.height, 1);
        assert_eq!(composed.desktop_origin, Point::new(-1.0, 0.0).unwrap());
        assert_eq!(
            composed.rgba,
            [
                255, 0, 0, 255, // Left display.
                0, 0, 0, 255, // Gap.
                0, 0, 255, 255, // Right display.
            ]
        );
    }

    #[test]
    fn composing_captures_scales_to_highest_pixel_density() {
        let low_density = CapturedPixels {
            rgba: vec![255, 0, 0, 255],
            width: 1,
            height: 1,
            desktop_origin: Point::new(0.0, 0.0).unwrap(),
            desktop_width: 1.0,
            desktop_height: 1.0,
        };
        let high_density = CapturedPixels {
            rgba: vec![0, 255, 0, 255, 0, 0, 255, 255],
            width: 2,
            height: 1,
            desktop_origin: Point::new(1.0, 0.0).unwrap(),
            desktop_width: 1.0,
            desktop_height: 1.0,
        };

        let composed = compose_captures(vec![low_density, high_density]).unwrap();

        assert_eq!(composed.width, 4);
        assert_eq!(composed.height, 2);
        assert_eq!(
            composed.rgba,
            [
                255, 0, 0, 255, 255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 0, 0, 255,
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255,
            ]
        );
    }

    #[test]
    fn composing_captures_rejects_invalid_pixel_data() {
        let invalid = CapturedPixels {
            rgba: vec![0, 0, 0],
            width: 1,
            height: 1,
            desktop_origin: Point::new(0.0, 0.0).unwrap(),
            desktop_width: 1.0,
            desktop_height: 1.0,
        };

        assert!(matches!(
            compose_captures(vec![invalid]),
            Err(Error::OperationFailed)
        ));
    }
}
