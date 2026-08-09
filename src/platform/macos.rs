use core_graphics::{
    access::ScreenCaptureAccess,
    color_space::CGColorSpace,
    context::CGContext,
    display::CGDisplay,
    event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton},
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::{CGPoint, CGRect, CGSize},
    image::{CGImageAlphaInfo, CGImageByteOrderInfo},
};
use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};

use crate::{Error, Point, Result, Screenshot};

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> u8;
}

pub(crate) struct Computer;

fn input_permission_granted() -> bool {
    // AXIsProcessTrusted has no arguments and returns the macOS Boolean type.
    unsafe { AXIsProcessTrusted() != 0 }
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

    pub(crate) fn screenshot(&self) -> Result<Screenshot> {
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
        Ok(Screenshot::new(
            png,
            width,
            height,
            desktop_origin,
            display_bounds.size.width,
            display_bounds.size.height,
        ))
    }
}
