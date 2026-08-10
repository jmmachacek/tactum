use std::{env, error::Error, fs, process, thread, time::Duration};

use tactum::Computer;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        exit_with_usage();
    };

    let computer = Computer::new()?;

    match command.as_str() {
        "screenshot" => {
            let Some(path) = args.next() else {
                exit_with_usage();
            };
            if args.next().is_some() {
                exit_with_usage();
            }

            let screenshot = computer.screenshot()?;
            fs::write(path, screenshot.png())?;
        }

        "move" | "click" => {
            let x = parse_coordinate(args.next(), "x");
            let y = parse_coordinate(args.next(), "y");
            if args.next().is_some() {
                exit_with_usage();
            }

            let screenshot = computer.screenshot()?;
            let point = screenshot.to_desktop_point(x, y)?;
            if command == "move" {
                computer.move_to(point)?;
            } else {
                computer.click(point)?;
            }
            // Keep the process alive long enough for macOS to deliver the event.
            thread::sleep(Duration::from_millis(20));
        }

        _ => exit_with_usage(),
    }

    Ok(())
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

fn exit_with_usage() -> ! {
    eprintln!("Usage:");
    eprintln!("  cargo run --example cli -- screenshot <path>");
    eprintln!("  cargo run --example cli -- move <image-x> <image-y>");
    eprintln!("  cargo run --example cli -- click <image-x> <image-y>");
    process::exit(2);
}
