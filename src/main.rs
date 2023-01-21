#![feature(async_closure)]

use std::{sync::Mutex, time::Duration};

use futures::executor::block_on;
use progress_view::{app::App, update::Update, widget::Widget};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let app = Mutex::new(App::new());
    let output = Mutex::new(0);

    {
        let mut app = app.lock().unwrap();
        app.add_widget(Widget::new_task("Header", 0));
        let widget = app.add_widget(Widget::new_task("Testing", 1));

        app.add_task(
            async move |s| {
                sleep(Duration::from_secs(2)).await;
                s.send(Update::set_message("weewoo")).await;

                return 2;
            },
            widget,
            output,
        );
    }

    let future = render_app(app);
    block_on(future);
}

pub async fn render_app(app: Mutex<App>) {
    let mut interval = tokio::time::interval(Duration::from_secs_f32(1.0 / 60.0));
    while !app.lock().unwrap().render() {
        interval.tick().await;
    }
}
