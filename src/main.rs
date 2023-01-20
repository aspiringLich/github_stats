#![feature(async_closure)]

use std::{sync::Mutex, io::stdout, time::Duration};

use crossterm::queue;
use progress_view::{app::App, widget::Widget};
use tokio::time::sleep;



extern crate tokio;



#[tokio::main]
async fn main() {
    let app = Mutex::new(App::new());
    let output = Mutex::new(0);

    {
        let mut app = app.lock().unwrap();
        let widget = app.add_widget(Widget::new_task("Testing"));
        app.add_widget(Widget::new_progress("this will never finish"));

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
    queue!(stdout(), crossterm::cursor::Hide).expect("no io errors");
    let mut interval = tokio::time::interval(Duration::from_secs_f32(1.0 / 60.0));
    while !app.lock().unwrap().render() {
        interval.tick().await;
    }
    queue!(stdout(), crossterm::cursor::Show).expect("no io errors");
}
