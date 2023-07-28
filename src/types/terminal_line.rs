use std::cmp::max;

use super::RStr;

use lazy_regex::lazy_regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalLine {
    /// text that displays while the line is focused
    focused_text: RStr,
    /// length of focused text to user (excludes ANSI escape codes)
    focused_vis_len: usize,
    /// text that displays while the line is unfocused
    unfocused_text: RStr,
    /// length of unfocused text to user (excludes ANSI escape codes)
    unfocused_vis_len: usize,
    /// id of the relevant html element. Used for `#id` links
    html_id: Option<RStr>,
    /// what happens when the user presses enter?
    interaction_type: InteractionType,
}

impl TerminalLine {
    /// pure fn to map displayed text
    pub fn map(self, f: impl Fn(RStr) -> RStr) -> Self {
        let focused_text = f(self.focused_text);
        let unfocused_text = f(self.unfocused_text);
        Self {
            focused_vis_len: get_visible_length(&focused_text),
            unfocused_vis_len: get_visible_length(&unfocused_text),
            focused_text,
            unfocused_text,
            ..self
        }
    }

    /// pure fn to map displayed text when not focused
    pub fn map_unfocused(self, f: impl Fn(RStr) -> RStr) -> Self {
        let unfocused_text = f(self.unfocused_text);
        Self {
            unfocused_vis_len: get_visible_length(&unfocused_text),
            unfocused_text,
            ..self
        }
    }

    /// pure fn to get the interaction type
    pub const fn interaction(&self) -> &InteractionType {
        &self.interaction_type
    }

    /// pure fn to display the line given whether or not it's focused
    pub fn display(&self, is_focused: bool) -> String {
        if is_focused {
            format!(">{}", self.focused_text)
        } else {
            format!(" {}", self.unfocused_text)
        }
    }

    /// pure fn to map displayed text when focused
    pub fn map_focused(self, f: impl Fn(RStr) -> RStr) -> Self {
        let focused_text = f(self.focused_text);
        Self {
            focused_vis_len: get_visible_length(&focused_text),
            focused_text,
            ..self
        }
    }

    /// pure fn to check if the line is empty
    pub fn is_empty(&self) -> bool {
        self.focused_text.is_empty() && self.unfocused_text.is_empty()
    }

    /// pure fn to set the interaction type
    #[allow(clippy::missing_const_for_fn)]
    pub fn with_interaction(self, interaction: InteractionType) -> Self {
        Self {
            interaction_type: interaction,
            ..self
        }
    }

    /// pure fn to set the html id
    #[allow(clippy::missing_const_for_fn)]
    pub fn with_id(self, id: RStr) -> Self {
        Self {
            html_id: Some(id),
            ..self
        }
    }

    /// pure fn to check if the id matches
    pub fn check_id(&self, id: &str) -> bool {
        self.html_id.as_ref().map_or(false, |str| id == &**str)
    }

    pub fn max_visible_length(&self) -> usize {
        max(self.focused_vis_len, self.unfocused_vis_len)
    }

    /// add spaces to the end of a line until its apparent size to the user (excluding ANSI escapes) matches the given value
    pub fn visible_right_pad(self, amount: usize) -> Self {
        assert!(amount >= self.focused_vis_len);
        assert!(amount >= self.unfocused_vis_len);
        Self {
            focused_text: format!(
                "{}{}",
                self.focused_text,
                " ".repeat(amount - self.focused_vis_len)
            )
            .into(),
            unfocused_text: format!(
                "{}{}",
                self.unfocused_text,
                " ".repeat(amount - self.unfocused_vis_len)
            )
            .into(),
            focused_vis_len: amount,
            unfocused_vis_len: amount,
            ..self
        }
    }
}

/// get the length of a str, ignoring ansi escape codes
fn get_visible_length(txt: &str) -> usize {
    lazy_regex!("\x1b\\[[\\d;]+m")
        .replace_all(txt, "")
        .to_string()
        .len()
}

impl From<RStr> for TerminalLine {
    fn from(value: RStr) -> Self {
        let vis_len = get_visible_length(&value);
        Self {
            focused_text: value.clone(),
            focused_vis_len: vis_len,
            unfocused_text: value,
            unfocused_vis_len: vis_len,
            interaction_type: InteractionType::None,
            html_id: None,
        }
    }
}

impl From<String> for TerminalLine {
    fn from(value: String) -> Self {
        Self::from(RStr::from(value))
    }
}

impl From<&str> for TerminalLine {
    fn from(value: &str) -> Self {
        Self::from(RStr::from(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractionType {
    // Input(String),
    // Toggle(bool),
    Image(RStr),
    Link(RStr),
    None,
}
