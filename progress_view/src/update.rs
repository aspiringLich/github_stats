
pub struct WidgetUpdate {
    update_type: Update,
    widget_index: usize,
}

pub enum Update {
    SetActive,
    SetDone,
    SetMessage(String),
}

impl WidgetUpdate {
    /// Creates a new update
    pub fn new(update_type: Update, widget_index: usize) -> Self {
        Self {
            update_type,
            widget_index,
        }
    }
}