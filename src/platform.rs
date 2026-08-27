use crate::Point;

fn coalesced_text_characters(text: &str) -> impl Iterator<Item = char> + '_ {
    let mut is_prev_cr = false;
    text.chars().filter(move |&c| {
        let emit = c != '\n' || !is_prev_cr;
        is_prev_cr = c == '\r';
        emit
    })
}

pub(crate) struct CapturedDisplay {
    pub(crate) id: u64,
    pub(crate) origin: Point,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) scale_x: f64,
    pub(crate) scale_y: f64,
}

pub(crate) struct CapturedScreenshot {
    pub(crate) png: Vec<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) desktop_origin: Point,
    pub(crate) desktop_width: f64,
    pub(crate) desktop_height: f64,
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(crate) use macos::Computer;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::Computer;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) use unsupported::Computer;

#[cfg(test)]
mod tests {
    use super::coalesced_text_characters;

    #[test]
    fn coalesces_crlf_without_changing_other_characters() {
        assert_eq!(
            coalesced_text_characters("a\r\nb\rc\nd\r\n\n世界").collect::<String>(),
            "a\rb\rc\nd\r\n世界"
        );
    }
}
