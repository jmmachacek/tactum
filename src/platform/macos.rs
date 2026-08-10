use core_graphics::{
    access::ScreenCaptureAccess,
    color_space::CGColorSpace,
    context::CGContext,
    display::CGDisplay,
    event::{CGEvent, CGEventTapLocation, CGEventType, CGKeyCode, CGMouseButton},
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::{CGPoint, CGRect, CGSize},
    image::{CGImageAlphaInfo, CGImageByteOrderInfo},
};
use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};

use crate::{Error, Key, Point, Result, platform::CapturedScreenshot};

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> u8;
}

pub(crate) struct Computer;

fn input_permission_granted() -> bool {
    // AXIsProcessTrusted has no arguments and returns the macOS Boolean type.
    unsafe { AXIsProcessTrusted() != 0 }
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

    pub(crate) fn click(&self, point: Point) -> Result<()> {
        if !input_permission_granted() {
            return Err(Error::PermissionDenied);
        }

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| Error::OperationFailed)?;
        let location = CGPoint::new(point.x(), point.y());
        let down = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::LeftMouseDown,
            location,
            CGMouseButton::Left,
        )
        .map_err(|_| Error::OperationFailed)?;
        let up = CGEvent::new_mouse_event(
            source,
            CGEventType::LeftMouseUp,
            location,
            CGMouseButton::Left,
        )
        .map_err(|_| Error::OperationFailed)?;

        down.post(CGEventTapLocation::HID);
        up.post(CGEventTapLocation::HID);
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

    pub(crate) fn screenshot(&self) -> Result<CapturedScreenshot> {
        if !ScreenCaptureAccess.preflight() {
            return Err(Error::PermissionDenied);
        }

        let display = CGDisplay::main();
        let display_bounds = display.bounds();
        let image = display.image().ok_or(Error::OperationFailed)?;
        let width = u32::try_from(image.width()).map_err(|_| Error::OperationFailed)?;
        let height = u32::try_from(image.height()).map_err(|_| Error::OperationFailed)?;
        let color_space = CGColorSpace::create_device_rgb();
        let bitmap_info = CGImageAlphaInfo::CGImageAlphaPremultipliedLast as u32
            | CGImageByteOrderInfo::CGImageByteOrder32Big as u32;
        let mut context = CGContext::create_bitmap_context(
            None,
            width as usize,
            height as usize,
            8,
            width as usize * 4,
            &color_space,
            bitmap_info,
        );
        let bounds = CGRect::new(
            &CGPoint::new(0.0, 0.0),
            &CGSize::new(width as f64, height as f64),
        );
        context.draw_image(bounds, &image);
        context.flush();

        let mut png = Vec::new();
        PngEncoder::new(&mut png)
            .write_image(context.data(), width, height, ExtendedColorType::Rgba8)
            .map_err(|_| Error::OperationFailed)?;

        let desktop_origin = Point::new(display_bounds.origin.x, display_bounds.origin.y)
            .map_err(|_| Error::OperationFailed)?;
        Ok(CapturedScreenshot {
            png,
            width,
            height,
            desktop_origin,
            desktop_width: display_bounds.size.width,
            desktop_height: display_bounds.size.height,
        })
    }
}
