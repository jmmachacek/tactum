use tactum::{Computer, Point};

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        exit_with_usage();
    };

    if command != "click" {
        exit_with_usage();
    }

    let x = parse_coordinate(args.next(), "x");
    let y = parse_coordinate(args.next(), "y");

    if args.next().is_some() {
        exit_with_usage();
    }

    let point = Point::new(x, y).expect("validated coordinates must form a point");
    let computer = Computer::new().unwrap_or_else(|error| exit_with_error(error));
    computer
        .click(point)
        .unwrap_or_else(|error| exit_with_error(error));
}

fn parse_coordinate(value: Option<String>, name: &str) -> f64 {
    value
        .and_then(|value| value.parse().ok())
        .filter(|value: &f64| value.is_finite() && *value >= 0.0)
        .unwrap_or_else(|| {
            eprintln!("{name} must be a non-negative number");
            exit_with_usage();
        })
}

fn exit_with_error(error: tactum::Error) -> ! {
    eprintln!("tactum: {error}");
    std::process::exit(1);
}

fn exit_with_usage() -> ! {
    eprintln!("Usage: tactum click <x> <y>");
    std::process::exit(2);
}
