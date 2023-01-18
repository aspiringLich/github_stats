#![feature(default_free_fn)]
#![feature(exclusive_range_pattern)]

use std::time::Instant;

use widget::Widget;

mod widget;

fn main() {
    for i in 0..=50 {
        let progress = Widget::new_progress("Testing testing");
        progress.update_progress(0.02 * i as f32);
        progress.render(Instant::now());
    }
}