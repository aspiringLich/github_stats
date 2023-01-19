#![feature(default_free_fn)]
#![feature(exclusive_range_pattern)]

use std::{time, default::default};

// use executor::Executor;
use widget::Widget;

mod widget;
mod app;

fn main() {
    for i in 0..=50 {
        let mut progress = Widget::new_progress("Testing testing");
        progress.update_progress(0.02 * i as f32);
        progress.render(time::SystemTime::now());
    }
    Widget::new_error("aaa").render(time::SystemTime::now());
}
