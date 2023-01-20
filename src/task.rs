// use std::{sync::{Mutex, Arc}, task::Poll, pin::Pin};

// use futures::{Future, future::IntoFuture};

// use crate::widget::Widget;


// pub struct WidgetTask<F, T>
// where
//     F: WidgetTaskFn<T>,
// {
//     f: F,
//     widget: Arc<Mutex<Widget>>,
//     out: Arc<Mutex<T>>,
// }

// impl<F: WidgetTaskFn<T>, T> WidgetTask<F, T> {
//     /// Create a new task
//     pub fn new(f: F, widget: Arc<Mutex<Widget>>, out: Arc<Mutex<T>>) -> Self {
//         Self { f, widget, out }
//     }
    
//     pub async fn run(self) {
//         let future = (self.f)(self.widget, self.out);
//         future.await
//     }
// }

// unsafe impl<F, T> Send for WidgetTask<F, T> where F: WidgetTaskFn<T> {}