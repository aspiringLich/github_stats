
#[derive(Debug, Clone)]
pub struct WidgetUpdate {
    pub update_type: Update,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub enum Update {
    SetActive,
    SetDone,
    SetMessage(String),
}

impl Update {
    pub fn set_message<T: Into<String>>(message: T) -> Self {
        Self::SetMessage(message.into())
    }
}

impl WidgetUpdate {
    /// Creates a new update
    pub fn new(update_type: Update, widget_index: usize) -> Self {
        Self {
            update_type,
            index: widget_index,
        }
    }
}
