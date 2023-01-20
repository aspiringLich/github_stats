#![feature(default_free_fn)]
#![feature(exclusive_range_pattern)]
#![feature(async_closure)]
#![feature(trait_alias)]

extern crate tokio;

use std::{default::default, sync::{RwLock, Mutex, Arc}, time::Duration, thread, borrow::BorrowMut};

use app::App;
// use executor::Executor;
use tokio::time::Interval;
use widget::Widget;

mod app;
mod widget;
mod repo;
mod task;

#[tokio::main]
async fn main() {
    let root = Widget::new_task("Testing");
    
    let app = Arc::new(Mutex::new(App::new()));
    app.lock().unwrap().add_widget(root);
    
    render_app(app).await;
}

pub async fn render_app(app: Arc<Mutex<App>>) {
    let mut interval = tokio::time::interval(Duration::from_secs_f32(1.0 / 30.0));

    dbg!("eee");
    while !app.lock().unwrap().render() {
        interval.tick().await;
    }
}
