//! Small pure functions used in Nasir

use crate::types::RStr;
use lazy_regex::lazy_regex;

pub fn get_link_destination(current: &str, link: &RStr) -> RStr {
    if link.starts_with("//") {
        format!("https:{link}").into()
    } else if link.starts_with('#') {
        format!("{current}{link}").into()
    } else if let Some(link) = link.strip_prefix('/') {
        format!(
            "{}{link}",
            &current
                .split('/')
                .take(3)
                .map(|s| format!("{s}/"))
                .collect::<String>()
        )
        .into()
    } else if let Some(link) = link.strip_prefix("..") {
        todo!("This is a backtracked relative link")
    } else if !link.contains('.') {
        todo!("This is a relative link")
    } else {
        link.clone()
    }
}

/// replace things like &#x24; and &lt;
pub fn transform_html_text(input: &str) -> RStr {
    // get hex entities
    let input = lazy_regex!("&#x([0-9a-fA-F]+);")
        .replace_all(input, |caps: &regex::Captures| {
            if let Some(hex_str) = caps.get(1) {
                if let Ok(codepoint) = u32::from_str_radix(hex_str.as_str(), 16) {
                    if let Some(character) = std::char::from_u32(codepoint) {
                        return character.to_string();
                    }
                }
            }
            caps[0].to_string() // Return the original match if conversion fails
        })
        .to_string();
    // get decimal entities
    lazy_regex!("&#(\\d+);")
        .replace_all(&input, |caps: &regex::Captures| {
            if let Some(dec_str) = caps.get(1) {
                if let Ok(codepoint) = dec_str.as_str().parse() {
                    if let Some(character) = std::char::from_u32(codepoint) {
                        return character.to_string();
                    }
                }
            }
            caps[0].to_string()
        })
        // get other entities
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&cent;", "¢")
        .replace("&pound;", "£")
        .replace("&yen;", "¥")
        .replace("&euro;", "€")
        .replace("&copy;", "©")
        .replace("&reg;", "®")
        .replace("&trade;", "™")
        .replace("&ndash;", "–")
        .replace("&mdash;", "—")
        .replace("&asymp;", "≈")
        .replace("&ne;", "≠")
        .replace("&deg;", "°")
        .into()
}

pub fn transform_url_text(input: &str) -> String {
    lazy_regex!("%([a-fA-F0-9]{2})")
        .replace_all(input, |caps: &regex::Captures| {
            String::from(
                // by regex, we know this is valid
                match i64::from_str_radix(caps.get(1).unwrap().as_str(), 16).unwrap() {
                    32 => " ",
                    33 => "!",
                    34 => "\"",
                    35 => "#",
                    36 => "$",
                    37 => "%",
                    38 => "&",
                    39 => "'",
                    40 => "(",
                    41 => ")",
                    42 => "*",
                    43 => "+",
                    44 => ",",
                    45 => "-",
                    46 => ".",
                    47 => "/",
                    _ => "",
                },
            )
        })
        .to_string()
}

/// transform rgb values to 8-bit colors
///
/// source: <https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797>
#[allow(clippy::cast_possible_truncation)]
pub const fn rgb_to_256((r, g, b): (usize, usize, usize)) -> u8 {
    let (r, g, b) = (
        ((r * 3) >> 7) as u8,
        ((g * 3) >> 7) as u8,
        ((b * 3) >> 7) as u8,
    );
    ((r * 36) + (g * 6) + b) + 16
}

pub fn wrap(txt: &str, width: usize) -> Vec<String> {
    let mut lines_buf = Vec::new();
    let mut current_line_buf = String::new();
    for w in txt.split_whitespace() {
        if w.len() >= width {
            if !current_line_buf.is_empty() {
                lines_buf.push(core::mem::take(&mut current_line_buf));
            }
            lines_buf.push(String::from(w));
            continue;
        }
        if current_line_buf.len() + w.len() >= width {
            lines_buf.push(core::mem::take(&mut current_line_buf));
        }
        current_line_buf.push(' ');
        current_line_buf.push_str(w);
    }
    if !current_line_buf.is_empty() {
        lines_buf.push(current_line_buf);
    }
    lines_buf
}

#[cfg(test)]
mod tests {
    use super::{get_link_destination, rgb_to_256, transform_html_text};

    #[test]
    fn transform() {
        assert_eq!(transform_html_text("&lt;&gt;"), "<>".into());
        assert_eq!(transform_html_text("&#60;&#62;"), "<>".into());
        assert_eq!(transform_html_text("&#xae;"), transform_html_text("&reg;"));
    }

    #[test]
    fn urls() {
        assert_eq!(
            get_link_destination("https://docs.rs/releases/2", &"/releases/3".into()),
            "https://docs.rs/releases/3".into()
        );
    }

    #[test]
    fn colors() {
        assert_eq!(rgb_to_256((0, 0, 0)), 16);
    }

    // #[test]
    // fn visible_length() {
    //     assert_eq!(get_visible_length("\x1b[31;5;1;1;1mHeyy\x1b[0m"), 4);
    // }
}
