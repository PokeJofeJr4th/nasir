use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub enum DocElement {
    HtmlElement {
        name: String,
        children: Vec<DocElement>,
        properties: BTreeMap<String, String>,
        inline: bool,
    },
    ClosingTag(String),
    Text(String),
}

impl DocElement {
    pub fn display(&self) -> String {
        match self {
            Self::HtmlElement {
                name,
                children,
                properties,
                inline,
            } => match name.as_ref() {
                "head" => String::new(),
                "img" => {
                    let alt = properties
                        .get("alt")
                        .map_or_else(|| properties.get("src").map_or("", |src| src), |alt| alt);
                    if alt.is_empty() {
                        String::from("[image]")
                    } else {
                        format!("[image: {alt}]")
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
                    str_buf
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
                    str_buf
                }
            },
            Self::Text(txt) => txt.clone(),
            Self::ClosingTag(_) => String::new(),
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
            Self::Text(txt) => Self::Text(txt.trim().to_string()),
            Self::ClosingTag(_) => Self::Text(String::new()),
        }
    }
}
