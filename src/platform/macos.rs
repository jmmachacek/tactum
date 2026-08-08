use core_graphics::{
    event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton},
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};

use crate::{Error, Point, Result};

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
        if !input_permission_granted() {
            return Err(Error::PermissionDenied);
        }

        Ok(Self)
    }

    pub(crate) fn click(&self, point: Point) -> Result<()> {
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| Error::EventSourceUnavailable)?;
        let location = CGPoint::new(point.x(), point.y());
        let down = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::LeftMouseDown,
            location,
            CGMouseButton::Left,
        )
        .map_err(|_| Error::EventCreationFailed)?;
        let up = CGEvent::new_mouse_event(
            source,
            CGEventType::LeftMouseUp,
            location,
            CGMouseButton::Left,
        )
        .map_err(|_| Error::EventCreationFailed)?;

        down.post(CGEventTapLocation::HID);
        up.post(CGEventTapLocation::HID);
        Ok(())
    }
}
