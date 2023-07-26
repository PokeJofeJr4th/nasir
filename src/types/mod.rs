mod doc_element;
mod terminal_line;

pub use prelude::*;

pub mod prelude {
    use std::rc::Rc;

    pub use super::doc_element::DocElement;
    pub use super::terminal_handler::TermHandler;
    pub use super::terminal_line::{InteractionType, TerminalLine};

    pub type RStr = Rc<str>;

    pub const SELF_CLOSING_TAGS: &[&str] = &["meta", "link", "image", "input", "img", "br"];
}

mod terminal_handler {
    use std::io::stdout;

    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };

    // DO NOT copy or clone
    pub struct TermHandler;

    impl TermHandler {
        pub fn new() -> Self {
            execute!(stdout(), EnterAlternateScreen).unwrap();
            enable_raw_mode().unwrap();
            Self
        }
    }

    impl Drop for TermHandler {
        fn drop(&mut self) {
            disable_raw_mode().unwrap();
            execute!(stdout(), LeaveAlternateScreen).unwrap();
        }
    }
}
