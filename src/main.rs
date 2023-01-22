#![feature(async_closure)]

use std::{sync::Mutex, time::Duration};

use futures::executor::block_on;
use progress_view::{app::{App, run_render}, update::Update, widget::Widget};
use tokio::{
    runtime::{Builder, Handle},
    time::sleep,
};


 fn main() {
    let mut app = App::default();
    let output = Mutex::new(0);

    {
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
    
    let mutex = Mutex::new(app);
    
    run_render(mutex);
}
