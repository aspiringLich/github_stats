use std::io::stdout;
use std::sync::Mutex;
use std::time::{self, Duration};

use crossterm::cursor::MoveUp;
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use futures::Future;
use tokio::runtime::{Builder, Handle, Runtime};

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::Interval;

use crate::update::{self, Update, WidgetUpdate};
use crate::widget::Widget;

pub struct App {
    pub widgets: Vec<Widget>,
    pub reciever: Receiver<update::WidgetUpdate>,
    pub sender: Sender<update::WidgetUpdate>,
    pub runtime: Runtime,
}

#[derive(Clone)]
pub struct UpdateSender {
    pub sender: Sender<update::WidgetUpdate>,
    pub index: usize,
}

impl UpdateSender {
    pub fn new(sender: Sender<update::WidgetUpdate>, index: usize) -> Self {
        Self { sender, index }
    }

    pub async fn send(&self, update: Update) {
        self.sender
            .send(WidgetUpdate::new(update, self.index))
            .await
            .expect("channel is open");
    }
}

impl Default for App {
    fn default() -> Self {
        let runtime = Builder::new_multi_thread().enable_all().build().unwrap();

        Self::new(runtime)
    }
}

impl App {
    pub fn new(runtime: Runtime) -> Self {
        let (sender, reciever) = mpsc::channel(64);

        Self {
            widgets: vec![],
            reciever,
            sender,
            runtime,
        }
    }

    /// adds a new task to the runtime
    pub fn add_task<E, F, T>(&mut self, f: E, index: usize, out: Mutex<T>)
    where
        E: FnOnce(UpdateSender) -> F + Send + 'static,
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let sender = UpdateSender::new(self.sender.clone(), index);
        self.runtime.spawn(async move {
            // set active to true so i get the cool spinner thingy
            sender.send(Update::SetActive).await;
            // run the actual task & pass it into output
            let ret = f(sender.clone()).await;
            *out.lock().unwrap() = ret;
            // set done
            sender.send(Update::SetDone).await;
        });
    }

    /// adds a widget to the app
    ///
    /// returns the index of the widget that you just added
    pub fn add_widget(&mut self, widget: Widget) -> usize {
        self.widgets.push(widget);
        return self.widgets.len() - 1;
    }

    /// does a recursive depth first search through the widget tree
    /// to display them & sets the widget as done if all of its children are done
    ///
    /// returns whether everything is done
    pub fn render(&mut self) -> bool {
        // handle all the updates
        while let Some(update) = self.reciever.try_recv().ok() {
            use Update::*;
            let widget = &mut self.widgets[update.index];

            match update.update_type {
                SetActive => {
                    widget.active = true;
                }
                SetDone => widget.set_done(),
                SetMessage(message) => {
                    widget.message = message;
                }
            }
        }

        // queue!(stdout(), crossterm::cursor::Hide).expect("no io err");
        let out = self.update_widget_status(0).all_done;
        self.render_widgets();

        out
    }

    fn render_widgets(&self) {
        let time = time::SystemTime::now();

        queue!(stdout(), MoveUp(self.widgets.len() as u16),).expect("no io err");

        for widget in &self.widgets {
            // put the backing behind it
            if widget.indent > 0 {
                queue!(
                    stdout(),
                    Clear(ClearType::CurrentLine),
                    Print(" ".repeat(widget.indent * 3 - 2) + "• ")
                )
                .expect("no io err")
            }

            widget.render(time);
        }
    }

    fn update_widget_status(&mut self, mut index: usize) -> RenderInner {
        let mut prev = index;
        let indent = self.widgets[index].indent;

        let mut out = RenderInner {
            new_index: index,
            all_done: true,
            active: false,
        };
        let mut children = false;

        macro ret() {{
            if children {
                let widget = &mut self.widgets[prev];
                if out.all_done {
                    widget.set_done()
                }
                if out.active {
                    widget.active = true;
                }
            }
            out
        }}

        while index < self.widgets.len() {
            let widget = &self.widgets[index];

            use std::cmp::Ordering::*;
            match widget.indent.cmp(&indent) {
                // if less, we should return
                Less => {
                    out.new_index = index;
                    return ret!();
                }
                // if equal, keep going
                Equal => {
                    if !widget.is_done() {
                        out.all_done = false;
                        // dbg!(&widget);
                    }
                    if widget.is_active() {
                        out.active = true;
                    }
                }
                // greater, this is out of our hands
                // we should call render_inner on this widget
                Greater => {
                    children = true;

                    let ret = self.update_widget_status(index);
                    // dbg!(&ret);

                    if ret.all_done {
                        self.widgets[prev].set_done();
                    } else {
                        out.all_done = false;
                    }
                    if ret.active {
                        self.widgets[prev].active = true;
                        out.active = true;
                    }

                    prev = index;
                    index = ret.new_index;
                }
            }

            index += 1;
        }
        return ret!();
    }
}

#[derive(Debug)]
struct RenderInner {
    new_index: usize,
    all_done: bool,
    active: bool,
}

pub fn run_render(app: Mutex<App>) {
    let mut app = app.lock().unwrap();
    print!("{}", "\n".repeat(app.widgets.len() + 1));

    let runtime = &app.runtime as *const Runtime as *mut Runtime;

    unsafe { runtime.as_mut().unwrap() }.block_on(async move {
        let mut interval = tokio::time::interval(Duration::from_secs_f32(0.1));

        while !app.render() {
            interval.tick().await;
        }
    });
    }
