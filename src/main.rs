#![feature(async_closure)]
#![feature(adt_const_params)]

use std::{sync::Mutex, time::Duration};

use progress_view::{app::{App, UpdateSender}, update::Update, widget::Widget};
use tokio::{
    time::sleep,
};

async fn update_message(sender: UpdateSender, message: &'static str, time: u64)  {
    sleep(Duration::from_secs(time)).await;
    sender.send(Update::set_message(message)).await;
}

 fn main() {
    let mut app = App::default();
    app.run_until_done();
}
