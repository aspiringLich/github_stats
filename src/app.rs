use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::time;

use crossterm::cursor::MoveUp;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{execute, queue};
use futures::Future;
use tokio::runtime::Runtime;

use crate::widget::Widget;

pub struct App {
    pub widgets: Vec<Arc<Mutex<Widget>>>,
    pub runtime: Runtime,
}

impl App {
    pub fn new() -> Self {
        Self {
            widgets: vec![],
            runtime: Runtime::new().unwrap(),
        }
    }

    /// adds a new task to the runtime
    pub fn add_task<E, F, T>(&mut self, f: E, widget: Arc<Mutex<Widget>>, out: Mutex<T>)
    where
        E: FnOnce(Arc<Mutex<Widget>>) -> F + Send + 'static,
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static + Sized,
    {
        // set active to true so i get the cool spinner thingy
        widget.lock().unwrap().active = true;
        self.runtime.spawn(async move {
            let ret = f(widget.clone()).await;
            *out.lock().unwrap() = ret;
            widget.lock().unwrap().set_done();
        });
    }

    /// adds a widget to the app
    pub fn add_widget(&mut self, widget: Widget) -> Arc<Mutex<Widget>> {
        self.widgets.push(Arc::new(Mutex::new(widget)));
        self.widgets.last().unwrap().clone()
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
        queue!(stdout(), Clear(ClearType::CurrentLine)).expect("no io err");
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
