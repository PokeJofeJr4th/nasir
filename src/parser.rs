use std::{collections::BTreeMap, iter::Peekable};

use lazy_regex::lazy_regex;

use crate::types::{DocElement, RStr, SELF_CLOSING_TAGS};

pub fn parse_html(html: &str) -> Result<DocElement, String> {
    let mut chars = if html.to_lowercase().starts_with("<!doctype html>") {
        &html[15..]
    } else {
        html
    }
    .chars()
    .peekable();
    consume_whitespace(&mut chars);
    inner_parse(&mut chars).map(DocElement::minify)
}

#[derive(PartialEq, Eq, Debug)]
enum TagType {
    Opening,
    Closing,
    DocType,
}

fn inner_parse<T: Iterator<Item = char>>(chars: &mut Peekable<T>) -> Result<DocElement, String> {
    match chars.next() {
        Some('<') => {
            let tag_type = match chars.peek() {
                Some('/') => {
                    chars.next();
                    TagType::Closing
                }
                Some('!') => {
                    chars.next();
                    TagType::DocType
                }
                _ => TagType::Opening,
            };
            let mut name_buf = String::new();
            consume_whitespace(chars);
            while let Some(c) = chars.peek() {
                if c.is_whitespace() {
                    chars.next();
                    break;
                } else if c.is_ascii_alphanumeric() || *c == '-' {
                    name_buf.push(*c);
                    chars.next();
                } else {
                    break;
                }
            }
            // no longer needs to be mutable
            let tag_name: RStr = name_buf.into();
            consume_whitespace(chars);
            match tag_type {
                TagType::Closing => {
                    if chars.next() == Some('>') {
                        Ok(DocElement::ClosingTag(tag_name))
                    } else {
                        Err(String::from("Missing `>`"))
                    }
                }
                TagType::DocType => {
                    let mut doctype_buf = String::new();
                    for c in chars.by_ref() {
                        if c == '>' {
                            // should consume
                            break;
                        }
                        doctype_buf.push(c);
                    }
                    if tag_name.starts_with("--") && doctype_buf.ends_with("--") {
                        Ok(DocElement::Text("".into()))
                    } else {
                        Ok(DocElement::ClosingTag(doctype_buf.into()))
                    }
                }
                TagType::Opening => get_opening_properties(chars, tag_name),
            }
        }
        Some(c) => {
            let mut txt_buf = String::from(c);
            while let Some(c) = chars.peek() {
                if *c == '<' {
                    break;
                }
                txt_buf.push(*c);
                chars.next();
            }
            Ok(DocElement::Text(transform_text(&txt_buf)))
        }
        None => Ok(DocElement::Text(RStr::from(""))),
    }
}

fn get_opening_properties<T: Iterator<Item = char>>(
    chars: &mut Peekable<T>,
    tag_name: RStr,
) -> Result<DocElement, String> {
    let mut props_buf = BTreeMap::new();
    let mut closed = false;
    'props: loop {
        if chars.peek() == Some(&'>') {
            chars.next();
            break;
        }
        let mut prop = String::new();
        while let Some(c) = chars.peek() {
            if c.is_alphanumeric() || *c == '-' {
                prop.push(*c);
                chars.next();
            } else if *c == '/' && prop.is_empty() {
                chars.next();
                closed = true;
                break;
            } else {
                break;
            }
        }
        consume_whitespace(chars);
        // if the next char isn't eq (eg `<script defer>`),
        // put it empty and continue
        let Some('=') = chars.peek() else {
                        props_buf.insert(prop.into(), RStr::from(""));
                        continue 'props;
                    };
        // consume `=`
        chars.next();
        consume_whitespace(chars);
        let mut content = String::new();
        if let Some(&barrier @ ('"' | '\'')) = chars.peek() {
            chars.next();
            while let Some(c) = chars.next() {
                if c == barrier {
                    break;
                } else if c == '\\' {
                    content.push(c);
                    let Some(c) = chars.next() else { return Err(String::from("Unexpected end of file")) };
                    content.push(c);
                } else {
                    content.push(c);
                }
            }
        } else {
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() {
                    content.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
        }
        consume_whitespace(chars);
        props_buf.insert(prop.into(), transform_text(&content));
    }
    // let Some('>') = chars.next() else { return Err(format!("Missing `>` for `<{tag_name}>`")) };
    // This is where it checks if you need a closing tag
    if SELF_CLOSING_TAGS.contains(&&*tag_name) | closed {
        Ok(DocElement::HtmlElement {
            name: tag_name,
            children: Vec::new(),
            properties: props_buf,
        })
    } else {
        // get children
        let mut children_buf = Vec::new();
        while chars.peek().is_some() {
            let child = inner_parse(chars)?;
            if child == DocElement::ClosingTag(tag_name.clone()) {
                break;
            }
            children_buf.push(child);
        }
        Ok(DocElement::HtmlElement {
            name: tag_name,
            children: children_buf,
            properties: props_buf,
        })
    }
}

fn consume_whitespace<T: Iterator<Item = char>>(chars: &mut Peekable<T>) {
    while let Some(c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            return;
        }
    }
}

fn transform_text(input: &str) -> RStr {
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
        .into()
}

#[cfg(test)]
mod tests {
    use super::transform_text;

    #[test]
    fn transform() {
        assert_eq!(transform_text("&lt;&gt;"), "<>".into());
        assert_eq!(transform_text("&#60;&#62;"), "<>".into());
        assert_eq!(transform_text("&#xae;"), transform_text("&reg;"));
    }
}
