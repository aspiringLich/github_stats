use std::sync::RwLock;

use crate::widget::{Widget, WidgetType};

pub struct App {
    pub widgets: Vec<RwLock<Widget>>,
}

impl App {
    pub fn new() -> Self {
        Self { widgets: vec![] }
    }

    pub fn add_widget(&mut self, widget: WidgetType) {
        self.widgets.push(RwLock::new(Widget::new(widget)));
    }
}
