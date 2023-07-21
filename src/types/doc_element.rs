use std::collections::BTreeMap;

use crossterm::terminal::SetTitle;

use super::{InteractionType, RStr, TerminalLine};

#[derive(Debug, PartialEq)]
pub enum DocElement {
    HtmlElement {
        name: RStr,
        children: Vec<DocElement>,
        properties: BTreeMap<RStr, RStr>,
    },
    ClosingTag(RStr),
    Text(RStr),
}

impl DocElement {
    pub fn display(&self, set_title: &mut SetTitle<RStr>) -> Vec<TerminalLine> {
        match self {
            Self::HtmlElement {
                name,
                children,
                properties,
            } => match name.as_ref() {
                // this should only set the title
                "head" => {
                    let mut children_buf: Vec<&Self> = children.iter().collect();
                    while let Some(child) = children_buf.pop() {
                        if let Self::HtmlElement {
                            name,
                            children,
                            properties: _,
                        } = child
                        {
                            if &**name == "title" {
                                let title = children
                                    .iter()
                                    // get the terminal lines
                                    .flat_map(|tl| tl.display(set_title))
                                    // get the text
                                    .map(|tl| tl.display(false))
                                    // make it into a string
                                    .map(|rstr| String::from(&*rstr))
                                    .collect::<String>();
                                set_title.0 = format!("{} - Nasir", title.trim()).into();
                            } else {
                                children_buf.extend(children);
                            }
                        }
                    }
                    Vec::new()
                }
                // this is for elements that shouldn't display anything under them
                "script" | "style" | "option" => Vec::new(),
                "img" => {
                    let alt = properties
                        .get("alt")
                        .map_or_else(|| properties.get("src").map_or("", |src| src), |alt| alt);
                    if alt.is_empty() {
                        vec![RStr::from("[image]").into()]
                    } else {
                        vec![RStr::from(format!("[image: {alt}]")).into()]
                    }
                }
                _ => {
                    let ret: Vec<TerminalLine> = children
                        .iter()
                        .flat_map(|tl| tl.display(set_title))
                        .filter(|tl| !tl.is_empty())
                        .collect();
                    let ret = match &**name {
                        "a" => {
                            let href: RStr = properties.get("href").map_or("".into(), Clone::clone);
                            ret.into_iter()
                                .map(|content| {
                                    // if it's already a link, prefer the lower-level one
                                    if let InteractionType::Link(_) = content.interaction() {
                                        content
                                    } else {
                                        content
                                            .map_focused(|str| {
                                                format!("({str})[\x1b[94m{href}\x1b[0m]").into()
                                            })
                                            .map_unfocused(|str| {
                                                format!("\x1b[4;94m{str}\x1b[0m").into()
                                            })
                                            .with_interaction(InteractionType::Link(href.clone()))
                                    }
                                })
                                .collect()
                        }
                        "b" | "strong" => ret
                            .into_iter()
                            .map(|tl| tl.map(|rstr| format!("\x1b[1m{rstr}\x1b[0m").into()))
                            .collect(),
                        "i" => ret
                            .into_iter()
                            .map(|tl| tl.map(|rstr| format!("\x3b[1m{rstr}\x1b[0m").into()))
                            .collect(),
                        _ => ret,
                    };
                    match properties.get("id") {
                        Some(id) => ret.into_iter().map(|tl| tl.with_id(id.clone())).collect(),
                        None => ret,
                    }
                }
            },
            Self::Text(txt) => vec![txt.clone().into()],
            Self::ClosingTag(_) => vec![RStr::from("").into()],
        }
    }

    pub fn minify(self) -> Self {
        match self {
            Self::HtmlElement {
                name,
                children,
                properties,
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
            },
            Self::Text(txt) => Self::Text(RStr::from(txt.trim())),
            Self::ClosingTag(_) => Self::Text(RStr::from("")),
        }
    }
}
