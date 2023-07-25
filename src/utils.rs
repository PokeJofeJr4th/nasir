//! Pure functions used in Nasir

use crate::types::RStr;
use lazy_regex::lazy_regex;

pub fn get_link_destination(current: &str, link: &RStr) -> RStr {
    if link.starts_with("//") {
        format!("https:{link}").into()
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
    } else {
        link.clone()
    }
}

pub fn transform_text(input: &str) -> RStr {
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

/// transform rgb values to 8-bit colors
/// 
/// source: <https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797>
#[allow(clippy::cast_possible_truncation)]
pub const fn rgb_to_256((r, g, b): (usize, usize, usize)) -> u8 {
    let (r, g, b) = (((r * 3) >> 7) as u8, ((g * 3) >> 7) as u8, ((b * 3) >> 7) as u8);
    ((r * 36) + (g * 6) + b) + 16
}

#[cfg(test)]
mod tests {
    use super::{get_link_destination, transform_text, rgb_to_256};

    #[test]
    fn transform() {
        assert_eq!(transform_text("&lt;&gt;"), "<>".into());
        assert_eq!(transform_text("&#60;&#62;"), "<>".into());
        assert_eq!(transform_text("&#xae;"), transform_text("&reg;"));
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
}
