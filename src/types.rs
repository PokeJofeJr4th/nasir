use std::{collections::BTreeMap, rc::Rc};

pub type RStr = Rc<str>;

#[derive(Debug, PartialEq)]
pub enum DocElement {
    HtmlElement {
        name: RStr,
        children: Vec<DocElement>,
        properties: BTreeMap<RStr, RStr>,
        inline: bool,
    },
    ClosingTag(RStr),
    Text(RStr),
}

impl DocElement {
    pub fn display(&self) -> RStr {
        match self {
            Self::HtmlElement {
                name,
                children,
                properties,
                inline,
            } => match name.as_ref() {
                "head" | "script" | "style" => RStr::from(""),
                "img" => {
                    let alt = properties
                        .get("alt")
                        .map_or_else(|| properties.get("src").map_or("", |src| src), |alt| alt);
                    if alt.is_empty() {
                        RStr::from("[image]")
                    } else {
                        RStr::from(format!("[image: {alt}]"))
                    }
                }
                "a" => {
                    let href = properties.get("href").map_or("", |href| href);
                    let mut str_buf = String::from('(');
                    for child in children {
                        let content = child.display();
                        if content.is_empty() || content.chars().all(char::is_whitespace) {
                            continue;
                        }
                        if !inline {
                            str_buf.push('\n');
                        }
                        str_buf.push_str(content.trim());
                    }
                    str_buf.push(')');
                    if !href.is_empty() {
                        str_buf.push_str(&format!("[{href}]"));
                    }
                    RStr::from(str_buf)
                }
                _ => {
                    let mut str_buf = String::new();
                    for child in children {
                        let content = child.display();
                        if content.is_empty() || content.chars().all(char::is_whitespace) {
                            continue;
                        }
                        if !inline {
                            str_buf.push('\n');
                        }
                        str_buf.push_str(child.display().trim());
                    }
                    RStr::from(str_buf)
                }
            },
            Self::Text(txt) => txt.clone(),
            Self::ClosingTag(_) => RStr::from(""),
        }
    }

    pub fn minify(self) -> Self {
        match self {
            Self::HtmlElement {
                name,
                children,
                properties,
                inline,
            } => Self::HtmlElement {
                name,
                // only pick the children that aren't empty
                children: children
                    .into_iter()
                    .map(Self::minify)
                    .filter(|html| match html {
                        Self::Text(txt) => !txt.trim().is_empty(),
                        _ => true,
                    })
                    .collect(),
                properties,
                inline,
            },
            Self::Text(txt) => Self::Text(RStr::from(txt.trim())),
            Self::ClosingTag(_) => Self::Text(RStr::from("")),
        }
    }
}
