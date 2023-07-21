use super::RStr;

pub struct TerminalLine {
    focused_text: RStr,
    unfocused_text: RStr,
    interaction_type: InteractionType,
}

impl TerminalLine {
    // pub fn map(self, f: impl Fn(RStr) -> RStr) -> Self {
    //     Self {
    //         focused_text: f(self.focused_text),
    //         unfocused_text: f(self.unfocused_text),
    //         interaction_type: self.interaction_type,
    //     }
    // }

    // pub fn map_unfocused(self, f: impl Fn(RStr) -> RStr) -> Self {
    //     Self {
    //         unfocused_text: f(self.unfocused_text),
    //         ..self
    //     }
    // }

    pub const fn interaction(&self) -> &InteractionType {
        &self.interaction_type
    }

    pub fn display(&self, is_focused: bool) -> RStr {
        if is_focused {
            self.focused_text.clone()
        } else {
            self.unfocused_text.clone()
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
}

impl From<RStr> for TerminalLine {
    fn from(value: RStr) -> Self {
        Self {
            focused_text: value.clone(),
            unfocused_text: value,
            interaction_type: InteractionType::None,
        }
    }
}

pub enum InteractionType {
    // Input(String),
    // Toggle(bool),
    Link(RStr),
    None,
}
