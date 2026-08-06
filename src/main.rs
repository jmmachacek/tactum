#[cfg(target_os = "macos")]
use core_graphics::{
    event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton},
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        show_usage()
    };

    if command != "click" {
        show_usage();
    }

    let x = parse_coordinate(args.next(), "x");
    let y = parse_coordinate(args.next(), "y");

    if args.next().is_some() {
        show_usage();
    }

    click(x, y);
}

fn parse_coordinate(value: Option<String>, name: &str) -> f64 {
    value
        .and_then(|value| value.parse().ok())
        .filter(|value: &f64| value.is_finite() && *value >= 0.0)
        .unwrap_or_else(|| {
            eprintln!("{name} must be a non-negative number");
            show_usage();
        })
}

#[cfg(target_os = "macos")]
fn click(x: f64, y: f64) {
    let point = CGPoint::new(x, y);
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .expect("macOS could not create an event source");
    let down = CGEvent::new_mouse_event(
        source.clone(),
        CGEventType::LeftMouseDown,
        point,
        CGMouseButton::Left,
    )
    .expect("macOS could not create a mouse-down event");
    let up = CGEvent::new_mouse_event(source, CGEventType::LeftMouseUp, point, CGMouseButton::Left)
        .expect("macOS could not create a mouse-up event");

    down.post(CGEventTapLocation::HID);
    up.post(CGEventTapLocation::HID);
}

#[cfg(not(target_os = "macos"))]
fn click(_: f64, _: f64) {
    eprintln!("tactum currently supports macOS only");
    std::process::exit(1);
}

fn show_usage() -> ! {
    eprintln!("Usage: tactum click <x> <y>");
    std::process::exit(2);
}
