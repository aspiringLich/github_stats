#![feature(default_free_fn)]
#![feature(exclusive_range_pattern)]

use std::{time, default::default};

// use executor::Executor;
use widget::Widget;

mod widget;
mod app;

fn main() {
    let mut root = Widget::new_task("Testing");
    
    for i in 0..10 {
        let mut widget = Widget::new_task(format!("Task {}", i));
        widget.set_done();
        root = root.add_child(widget);
    }

    let mut app = app::App::new();
    app.add_widget(root);
    app.render();
    app.render();
    loop {}
}
