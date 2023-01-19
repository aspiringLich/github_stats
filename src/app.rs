use std::{sync::RwLock, io::stdout};
use std::time;

use crossterm::cursor::{RestorePosition, MoveUp};
use crossterm::{cursor::SavePosition, execute};

use crate::widget::{self, Widget, WidgetType};

pub struct App {
    pub widgets: Vec<RwLock<Widget>>,
}

impl App {
    pub fn new() -> Self {
        Self { widgets: vec![] }
    }

    /// adds a widget to the app
    pub fn add_widget(&mut self, widget: Widget) {
        self.widgets.push(RwLock::new(widget));
    }

    /// does a recursive depth first search through the widget tree
    /// to display them & sets the widget as done if all of its children are done
    pub fn render(&mut self) {
        let time = time::SystemTime::now();
        
        let mut lines = 0;
        for widget in self.widgets.iter_mut() {
            Self::_render_inner(widget, time, 0, &mut lines);
        }
        execute!(stdout(), MoveUp(lines)).expect("no io errors");
    }

    fn _render_inner(
        widget: &mut RwLock<Widget>,
        time: time::SystemTime,
        depth: usize,
        lines: &mut u16
    ) -> (bool, bool) {
        let mut w = widget.write().unwrap();

        // print out stuff for this widget
        w.render(time);
        *lines += 1;

        // do the actual search
        let (mut done, mut active) = (false, false);
        let len = w.children.len();
        for (i, child) in w.children.iter_mut().enumerate() {
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
