use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time;
use std::{io::stdout, sync::RwLock};

use crossterm::cursor::MoveUp;
use crossterm::execute;
use futures::Future;
use tokio::runtime::Runtime;

use crate::widget::{self, Widget, WidgetType};

pub struct App {
    pub widgets: Vec<Arc<Mutex<Widget>>>,
    pub runtime: Runtime,
}

trait WidgetTaskFn<T> =
    FnOnce(Arc<Mutex<Widget>>, Arc<Mutex<T>>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send;


impl App {
    pub fn new() -> Self {
        Self { widgets: vec![], runtime: Runtime::new().unwrap() }
    }
    
    /// adds a new task to the runtime
    pub fn add_task<F, T>(&mut self, f: F, widget: Arc<Mutex<Widget>>, out: Arc<Mutex<T>>)
    where
        F: WidgetTaskFn<T>,
        T: Send + Sync + 'static,
    {
        self.runtime.spawn(async move {
            f(widget, out).await
        });
    }

    /// adds a widget to the app
    pub fn add_widget(&mut self, widget: Widget) {
        self.widgets.push(Arc::new(Mutex::new(widget)));
    }

    /// does a recursive depth first search through the widget tree
    /// to display them & sets the widget as done if all of its children are done
    pub fn render(&mut self) -> bool {
        let time = time::SystemTime::now();

        let mut lines = 0;
        let mut out = true;
        for widget in self.widgets.iter() {
            let (done, _) = Self::_render_inner(widget, time, 0, &mut lines);
            
            if !done {
                out = false;
            }
        }
        execute!(stdout(), MoveUp(lines)).expect("no io errors");
        out
    }

    fn _render_inner(
        widget: &Mutex<Widget>,
        time: time::SystemTime,
        depth: usize,
        lines: &mut u16,
    ) -> (bool, bool) {
        let mut w = widget.lock().unwrap();

        // print out stuff for this widget
        w.render(time);
        *lines += 1;

        // do the actual search
        let (mut done, mut active) = (false, false);
        let len = w.children.len();
        for (i, child) in w.children.iter().enumerate() {
            // render the indentation before the child widgets render themselves
            if i == 0 {
                print!("{} └┬─", " ".repeat(depth * 4));
            } else {
                let ch = if i == len - 1 { '└' } else { '├' };
                print!("{}{}─", " ".repeat(depth * 4 + 2), ch);
            }

            // if any are done or active, set done or active
            let (_done, _active) = Self::_render_inner(child, time, depth + 1, lines);
            if _done {
                done = true
            };
            if _active {
                active = true
            };
        }
        // set it done if all of its children are done
        if done {
            w.set_done();
        }
        // take a guess
        if active {
            w.active = true;
        }
        (w.is_done(), w.is_active())
    }
}
