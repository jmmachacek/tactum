use core_graphics::{
    event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton},
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};

use crate::{Error, Point, Result};

pub(crate) struct Computer;

impl Computer {
    pub(crate) fn new() -> Result<Self> {
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
