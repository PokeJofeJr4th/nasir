use std::{
    cmp::max,
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crossterm::terminal::{self, SetTitle};

use crate::{
    cacher::{self, ByteCacher},
    get_link_destination, img,
    utils::wrap,
};

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
    pub fn display(
        &self,
        set_title: &mut SetTitle<RStr>,
        cacher: &Arc<Mutex<ByteCacher>>,
        link: &str,
        verbose: bool,
    ) -> Vec<TerminalLine> {
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
                                    .flat_map(|tl| tl.display(set_title, cacher, link, verbose))
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
                "img" => display_img(properties, cacher, link, verbose),
                _ => {
                    let ret: Vec<TerminalLine> = children
                        .iter()
                        .flat_map(|tl| tl.display(set_title, cacher, link, verbose))
                        .filter(|tl| !tl.is_empty())
                        .collect();
                    let ret = display_formatted_element(name, properties, ret);
                    match properties.get("id") {
                        Some(id) => ret.into_iter().map(|tl| tl.with_id(id.clone())).collect(),
                        None => ret,
                    }
                }
            },
            Self::Text(txt) => wrap(txt, (terminal::size().unwrap().0 - 1) as usize)
                .into_iter()
                .map(|str| TerminalLine::from(&str[1..]))
                .collect(),
            Self::ClosingTag(_) => vec![RStr::from("").into()],
        }
    }

    /// pure function to collapse some elements and so on
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

/// pure function to apply special formatting to the output of `DocElement::display`
fn display_formatted_element(
    name: &str,
    properties: &BTreeMap<RStr, RStr>,
    ret: Vec<TerminalLine>,
) -> Vec<TerminalLine> {
    match name {
        "a" => {
            let href: RStr = properties.get("href").map_or("".into(), Clone::clone);
            ret.into_iter()
                .map(|content| {
                    // if it's already a link, prefer the lower-level one
                    if let InteractionType::Link(_) = content.interaction() {
                        content
                    } else {
                        content
                            // >(text)[link] when focused
                            .map_focused(|str| format!("({str})[\x1b[94m{href}\x1b[0m]").into())
                            // blue underlined link when unfocused
                            .map_unfocused(|str| format!("\x1b[4;94m{str}\x1b[0m").into())
                            .with_interaction(InteractionType::Link(href.clone()))
                    }
                })
                .collect()
        }
        // bold font face
        "b" | "strong" => ret
            .into_iter()
            .map(|tl| tl.map(|rstr| format!("\x1b[1m{rstr}\x1b[0m").into()))
            .collect(),
        // light font face
        "i" => ret
            .into_iter()
            .map(|tl| tl.map(|rstr| format!("\x1b[1m{rstr}\x1b[0m").into()))
            .collect(),
        // get max width, dull colors for code blocks
        "code" => {
            let width = ret
                .iter()
                .map(|tl| max(tl.display(false).len(), tl.display(true).len()) - 1)
                .max()
                .unwrap_or(0);
            ret.into_iter()
                .map(|tl| {
                    tl.map(|rstr| format!("\x1b[38;5;250;48;5;240m{rstr:width$}\x1b[0m").into())
                })
                .collect()
        }
        "h1" => {
            let width = ret
                .iter()
                .map(|tl| max(tl.display(false).len(), tl.display(true).len()) - 1)
                .max()
                .unwrap_or(0);
            let mut buf = vec![TerminalLine::from(format!("╔{:═<width$}╗", ""))];
            buf.extend(
                ret.into_iter()
                    .map(|tl| tl.map(|rstr| format!("║\x1b[30;47m{rstr:width$}\x1b[0m║").into())),
            );
            buf.push(TerminalLine::from(format!("╚{:═<width$}╝", "")));
            buf
        }
        "h2" => {
            let width = ret
                .iter()
                .map(|tl| max(tl.display(false).len(), tl.display(true).len()) - 1)
                .max()
                .unwrap_or(0);
            let mut buf = vec![TerminalLine::from(format!("╔{:═<width$}╗", ""))];
            buf.extend(
                ret.into_iter()
                    .map(|tl| tl.map(|rstr| format!("║{rstr:width$}║").into())),
            );
            buf.push(TerminalLine::from(format!("╚{:═<width$}╝", "")));
            buf
        }
        "h3" => {
            let width = ret
                .iter()
                .map(|tl| max(tl.display(false).len(), tl.display(true).len()) - 1)
                .max()
                .unwrap_or(0);
            let mut buf = vec![TerminalLine::from(format!("┌{:─<width$}┐", ""))];
            buf.extend(
                ret.into_iter()
                    .map(|tl| tl.map(|rstr| format!("│{rstr:width$}│").into())),
            );
            buf.push(TerminalLine::from(format!("└{:─<width$}┘", "")));
            buf
        }
        "h4" => ret
            .into_iter()
            .map(|tl| tl.map(|rstr| format!("\x1b[30;47m{rstr}\x1b[0m").into()))
            .collect(),
        "h5" => ret
            .into_iter()
            .map(|tl| tl.map(|rstr| format!("#### {rstr} ####").into()))
            .collect(),
        "h6" => ret
            .into_iter()
            .map(|tl| tl.map(|rstr| format!("## {rstr} ##").into()))
            .collect(),
        _ => ret,
    }
}

fn display_img(
    properties: &BTreeMap<RStr, RStr>,
    cacher: &Arc<Mutex<ByteCacher>>,
    base_link: &str,
    verbose: bool,
) -> Vec<TerminalLine> {
    let src = properties.get("src");
    src.and_then(|src| {
        cacher::get_from_cache(
            cacher.clone(),
            &get_link_destination(base_link, src),
            verbose,
        )
    })
    .and_then(|img_bytes| {
        // if we can get an image, return it
        if verbose {
            print!("`display_img` Got something from cache\r\n");
        }
        img::get_image(&img_bytes).map_or(None, |img| {
            if verbose {
                print!("The thing `display_img` got from cache worked\r\n");
            }
            let img = img::approximate_image(
                &img,
                {
                    let size = terminal::size().unwrap();
                    ((size.0 / 2).into(), (size.1 / 2).into())
                },
                verbose,
            );
            if let Some(src) = src {
                Some(
                    img.into_iter()
                        .map(|tl| tl.with_interaction(InteractionType::Image(src.clone())))
                        .collect(),
                )
            } else {
                Some(img)
            }
        })
    })
    .unwrap_or_else(|| {
        let alt = properties
            .get("alt")
            .map_or_else(|| src.map_or("", |src| src), |alt| alt);
        if alt.is_empty() {
            vec![RStr::from("[image]").into()]
        } else {
            vec![RStr::from(format!("[image: {alt}]")).into()]
        }
    })
}
