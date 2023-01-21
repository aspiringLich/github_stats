use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::time;

use crossterm::cursor::{MoveUp, SavePosition, RestorePosition};
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{execute, queue};
use futures::Future;
use tokio::runtime::Runtime;

use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::update::{self, WidgetUpdate, Update};
use crate::widget::Widget;

pub struct App {
    pub widgets: Vec<Widget>,
    pub runtime: Runtime,
    pub reciever: Receiver<update::WidgetUpdate>,
    pub sender: Sender<update::WidgetUpdate>,
}

#[derive(Clone)]
pub struct UpdateSender {
    pub sender: Sender<update::WidgetUpdate>,
    pub index: usize
}

impl UpdateSender {
    pub fn new(sender: Sender<update::WidgetUpdate>, index: usize) -> Self {
        Self {
            sender,
            index
        }
    }
    
    pub async fn send(&self, update: Update) {
        self.sender.send(WidgetUpdate::new(update, self.index)).await;
    }
}

impl App {
    pub fn new() -> Self {
        let (sender, reciever) = mpsc::channel(64);
        
        Self {
            widgets: vec![],
            runtime: Runtime::new().unwrap(),
            reciever,
            sender,
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
        let time = time::SystemTime::now();
        
        let out = Self::render_inner(0, &self.widgets, time).all_done;
        execute!(stdout(), MoveUp(self.widgets.len() as u16)).expect("no io err");
        
        out
    }

    fn render_inner(
        index: usize,
        widgets: &Vec<Widget>,
        time: time::SystemTime,
    ) -> RenderInner {
        let indent = widgets[index].indent;
        
        let mut out = RenderInner {
            new_index: index,
            all_done: true,
            active: false,
        };
        
        for index in index..widgets.len() {
            let widget = &widgets[index];
            
            use std::cmp::Ordering::*;
            match widget.indent.cmp(&indent) {
                // if less, we should return
                Less => {
                    out.new_index = index;
                    return out;
                },
                // if equal, keep going
                Equal => {
                    if !widget.is_done() {
                        out.all_done = false;
                    }
                    if widget.is_active() {
                        out.active = true;
                    }
                    // put the backing behind it
                    if indent > 0 {
                        queue!(stdout(), Clear(ClearType::CurrentLine), Print(" ".repeat(indent * 3 - 2) + "• ")).expect("no io err")
                    }
                    
                    widget.render(time);
                },
                // greater, this is out of our hands
                // we should call render_inner on this widget
                Greater => {
                    let inner = Self::render_inner(index, widgets, time);
                    if !inner.all_done {
                        out.all_done = false;
                    }
                    if inner.active {
                        out.active = true;
                    }
                    out.new_index = inner.new_index;
                },
            }
        }
        
        out
    }
}

struct RenderInner {
    new_index: usize,
    all_done: bool,
    active: bool,
}
