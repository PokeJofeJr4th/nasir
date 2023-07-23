use super::RStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalLine {
    focused_text: RStr,
    unfocused_text: RStr,
    html_id: Option<RStr>,
    interaction_type: InteractionType,
}

impl TerminalLine {
    pub fn map(self, f: impl Fn(RStr) -> RStr) -> Self {
        Self {
            focused_text: f(self.focused_text),
            unfocused_text: f(self.unfocused_text),
            ..self
        }
    }

    pub fn map_unfocused(self, f: impl Fn(RStr) -> RStr) -> Self {
        Self {
            unfocused_text: f(self.unfocused_text),
            ..self
        }
    }

    pub const fn interaction(&self) -> &InteractionType {
        &self.interaction_type
    }

    pub fn display(&self, is_focused: bool) -> String {
        if is_focused {
            format!(">{}", self.focused_text)
        } else {
            format!(" {}", self.unfocused_text)
        }
    }

    pub fn map_focused(self, f: impl Fn(RStr) -> RStr) -> Self {
        Self {
            focused_text: f(self.focused_text),
            ..self
        }
    }

    pub fn is_empty(&self) -> bool {
        self.focused_text.is_empty() && self.unfocused_text.is_empty()
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn with_interaction(self, interaction: InteractionType) -> Self {
        Self {
            interaction_type: interaction,
            ..self
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn with_id(self, id: RStr) -> Self {
        Self {
            html_id: Some(id),
            ..self
        }
    }

    // pub fn check_id<T: PartialEq<RStr>>(&self, id: T) -> bool {
    //     self.html_id.as_ref().map_or(false, |str| id == str.clone())
    // }
}

impl From<RStr> for TerminalLine {
    fn from(value: RStr) -> Self {
        Self {
            focused_text: value.clone(),
            unfocused_text: value,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractionType {
    // Input(String),
    // Toggle(bool),
    Link(RStr),
    None,
}
