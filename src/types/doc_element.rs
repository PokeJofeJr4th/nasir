use std::collections::BTreeMap;

use super::{RStr, TerminalLine};

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
    pub fn display(&self) -> Vec<TerminalLine> {
        match self {
            Self::HtmlElement {
                name,
                children,
                properties,
            } => match name.as_ref() {
                // this is for elements that shouldn't display anything under them
                "head" | "script" | "style" | "option" => Vec::new(),
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
                "a" => {
                    let href: RStr = properties.get("href").map_or("".into(), Clone::clone);
                    children
                        .iter()
                        .flat_map(Self::display)
                        .filter(|tl| !tl.is_empty())
                        .map(|content| {
                            content
                                .map_focused(|str| format!("({str})[{href}]").into())
                                .with_interaction(super::InteractionType::Link(href.clone()))
                        })
                        .collect()
                }
                _ => children
                    .iter()
                    .flat_map(Self::display)
                    .filter(|tl| !tl.is_empty())
                    .collect(),
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
