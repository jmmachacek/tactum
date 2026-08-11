use std::{env, error::Error, fs, process, thread, time::Duration};

use tactum::{Computer, Key, MouseButton};

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

        "move" => {
            let x = parse_coordinate(args.next(), "x");
            let y = parse_coordinate(args.next(), "y");
            if args.next().is_some() {
                exit_with_usage();
            }

            let screenshot = computer.screenshot()?;
            let point = screenshot.to_desktop_point(x, y)?;
            computer.move_to(point)?;
        }

        "click" | "double-click" => {
            let Some(button) = args.next().as_deref().and_then(parse_mouse_button) else {
                exit_with_usage();
            };
            let x = parse_coordinate(args.next(), "x");
            let y = parse_coordinate(args.next(), "y");
            if args.next().is_some() {
                exit_with_usage();
            }

            let screenshot = computer.screenshot()?;
            let point = screenshot.to_desktop_point(x, y)?;
            if command == "click" {
                computer.click(button, point)?;
            } else {
                computer.double_click(button, point)?;
            }
        }

        "drag" => {
            let Some(button) = args.next().as_deref().and_then(parse_mouse_button) else {
                exit_with_usage();
            };
            let from_x = parse_coordinate(args.next(), "from-x");
            let from_y = parse_coordinate(args.next(), "from-y");
            let to_x = parse_coordinate(args.next(), "to-x");
            let to_y = parse_coordinate(args.next(), "to-y");
            if args.next().is_some() {
                exit_with_usage();
            }

            let screenshot = computer.screenshot()?;
            let from = screenshot.to_desktop_point(from_x, from_y)?;
            let to = screenshot.to_desktop_point(to_x, to_y)?;
            computer.drag(button, from, to)?;
        }

        "scroll" => {
            let horizontal = parse_int(args.next(), "horizontal");
            let vertical = parse_int(args.next(), "vertical");
            if args.next().is_some() {
                exit_with_usage();
            }

            computer.scroll(horizontal, vertical)?;
        }

        "key" => {
            let Some(key) = args.next().as_deref().and_then(parse_key) else {
                exit_with_usage();
            };
            if args.next().is_some() {
                exit_with_usage();
            }

            computer.key_press(key)?;
        }

        "type" => {
            let Some(text) = args.next() else {
                exit_with_usage();
            };
            if args.next().is_some() {
                exit_with_usage();
            }

            computer.type_text(&text)?;
        }

        "read-clipboard" => {
            if args.next().is_some() {
                exit_with_usage();
            }

            if let Some(text) = computer.read_clipboard()? {
                println!("{text}");
            }
        }

        "write-clipboard" => {
            let Some(text) = args.next() else {
                exit_with_usage();
            };
            if args.next().is_some() {
                exit_with_usage();
            }

            computer.write_clipboard(&text)?;
        }

        _ => exit_with_usage(),
    }

    // Keep the process alive long enough for the OS to deliver the event(s).
    thread::sleep(Duration::from_millis(20));

    Ok(())
}

fn parse_int(value: Option<String>, name: &str) -> i32 {
    value
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| {
            eprintln!("{name} must be an integer");
            exit_with_usage();
        })
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

fn parse_mouse_button(value: &str) -> Option<MouseButton> {
    match value {
        "left" => Some(MouseButton::Left),
        "right" => Some(MouseButton::Right),
        "middle" => Some(MouseButton::Middle),
        _ => None,
    }
}

fn parse_key(value: &str) -> Option<Key> {
    match value {
        "a" => Some(Key::A),
        "b" => Some(Key::B),
        "c" => Some(Key::C),
        "d" => Some(Key::D),
        "e" => Some(Key::E),
        "f" => Some(Key::F),
        "g" => Some(Key::G),
        "h" => Some(Key::H),
        "i" => Some(Key::I),
        "j" => Some(Key::J),
        "k" => Some(Key::K),
        "l" => Some(Key::L),
        "m" => Some(Key::M),
        "n" => Some(Key::N),
        "o" => Some(Key::O),
        "p" => Some(Key::P),
        "q" => Some(Key::Q),
        "r" => Some(Key::R),
        "s" => Some(Key::S),
        "t" => Some(Key::T),
        "u" => Some(Key::U),
        "v" => Some(Key::V),
        "w" => Some(Key::W),
        "x" => Some(Key::X),
        "y" => Some(Key::Y),
        "z" => Some(Key::Z),
        "backspace" => Some(Key::Backspace),
        "tab" => Some(Key::Tab),
        "return" => Some(Key::Return),
        "escape" => Some(Key::Escape),
        "space" => Some(Key::Space),
        "delete" => Some(Key::Delete),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "page-up" => Some(Key::PageUp),
        "page-down" => Some(Key::PageDown),
        "left" => Some(Key::Left),
        "right" => Some(Key::Right),
        "down" => Some(Key::Down),
        "up" => Some(Key::Up),
        "shift" => Some(Key::Shift),
        "control" => Some(Key::Control),
        "option" => Some(Key::Option),
        "command" => Some(Key::Command),
        _ => None,
    }
}

fn exit_with_usage() -> ! {
    eprintln!("Usage:");
    eprintln!("  cargo run --example cli -- screenshot <path>");
    eprintln!("  cargo run --example cli -- move <image-x> <image-y>");
    eprintln!("  cargo run --example cli -- click <button> <image-x> <image-y>");
    eprintln!("  cargo run --example cli -- double-click <button> <image-x> <image-y>");
    eprintln!("  cargo run --example cli -- drag <button> <from-x> <from-y> <to-x> <to-y>");
    eprintln!("  cargo run --example cli -- scroll <horizontal> <vertical>");
    eprintln!("  cargo run --example cli -- key <key>");
    eprintln!("  cargo run --example cli -- type <text>");
    eprintln!("  cargo run --example cli -- read-clipboard");
    eprintln!("  cargo run --example cli -- write-clipboard <text>");
    process::exit(2);
}
