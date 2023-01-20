#![feature(default_free_fn)]
#![feature(exclusive_range_pattern)]
#![feature(async_closure)]
#![feature(trait_alias)]

extern crate tokio;

use std::{
    borrow::BorrowMut,
    default::default,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Duration,
};

use app::App;
// use executor::Executor;
use tokio::time::{sleep, Interval};
use widget::{Widget, WidgetType};

mod app;
mod repo;
mod task;
mod widget;

#[tokio::main]
async fn main() {
    let app = Mutex::new(App::new());
    let output = Mutex::new(0);

    {
        let mut app = app.lock().unwrap();
        let widget = app.add_widget(Widget::new_task("Testing"));

        app.add_task(
            async move |w| {
                sleep(Duration::from_secs(2)).await;
                let mut w = w.lock().unwrap();
                w.set_message("eieio");
                return 1;
            },
            widget,
            output,
        );
    }

    render_app(app).await;
}

pub async fn render_app(app: Mutex<App>) {
    let mut interval = tokio::time::interval(Duration::from_secs_f32(1.0 / 30.0));
    while !app.lock().unwrap().render() {
        interval.tick().await;
    }
}
