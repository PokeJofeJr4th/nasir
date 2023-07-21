mod doc_element;
mod terminal_line;

pub use prelude::*;

pub mod prelude {
    use std::rc::Rc;

    pub use super::doc_element::DocElement;
    pub use super::terminal_line::{InteractionType, TerminalLine};

    pub type RStr = Rc<str>;

    pub const SELF_CLOSING_TAGS: &[&str] = &["meta", "link", "image", "input"];
}
