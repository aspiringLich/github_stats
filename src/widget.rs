use crossterm::style::{self, Stylize};

use std::{default::default, sync::RwLock, time};

#[derive(Clone, Debug, Default)]
pub enum WidgetType {
    #[default]
    None,
    // {message}
    Text {
        message: &'static str,
    },
    // [⠦] [━━━━━━━━[ 55.0%]        ] {message}
    Percentage {
        message: &'static str,
        progress: f32,
    },
    // [⠦] [━━━━━╸  [ 5/20]         ] {message}
    Progress {
        message: &'static str,
        progress: usize,
        total: usize,
    },
    // [⠦] {message}
    Task {
        message: &'static str,
        done: bool,
    },
    // [⚠️] {message}
    Error {
        message: &'static str,
    },
}

#[derive(Default)]
pub struct Widget {
    pub widget: WidgetType,
    pub children: Vec<RwLock<Widget>>,
    pub active: bool,
}

impl Widget {
    /// creates a widget from a widget type
    pub fn new(widget: WidgetType) -> Self {
        Self {
            widget,
            ..default()
        }
    }

    /// Create a new text widget
    pub fn new_text(message: &'static str) -> Self {
        Self {
            widget: WidgetType::Text { message },
            ..default()
        }
    }

    /// Create a new progress widget
    pub fn new_progress(message: &'static str) -> Self {
        Self {
            widget: WidgetType::Percentage {
                message,
                progress: 0.0,
            },
            ..default()
        }
    }

    /// Create a new discrete progress widget
    pub fn new_discrete_progress(message: &'static str, total: usize) -> Self {
        Self {
            widget: WidgetType::Progress {
                message,
                progress: 0,
                total,
            },
            ..default()
        }
    }

    /// Create a new error widget
    pub fn new_error(message: &'static str) -> Self {
        Self {
            widget: WidgetType::Error { message },
            ..default()
        }
    }

    /// Update the progress of a progress widget
    pub fn update_progress(&mut self, progress: f32) {
        if let WidgetType::Percentage { progress: ref mut p, .. } = self.widget {
            *p = progress;
        }
    }

    /// Update the progress of a discrete progress widget
    pub fn update_discrete_progress(&mut self, progress: usize) {
        if let WidgetType::Progress { progress: ref mut p, .. } = self.widget {
            *p = progress;
        }
    }

    /// Update whether a task is done
    pub fn update_task_done(&mut self, done: bool) {
        if let WidgetType::Task { done: ref mut d, .. } = self.widget {
            *d = done;
        }
    }

    /// adds a child to this widget tree
    pub fn add_child(mut self, widget: WidgetType) -> Self {
        self.children.push(RwLock::new(Widget::new(widget)));
        self
    }

    /// adds children to this widget tree
    pub fn add_children(mut self, children: &[WidgetType]) -> Self {
        self.children
            .extend(children.iter().map(|w| RwLock::new(Widget::new(w.clone()))));
        self
    }
    
    /// detects if this widget is done
    pub fn is_done(&self) -> bool {
        use WidgetType::*;
        match self.widget {
            None => true,
            Text { .. } => true,
            Percentage { progress, .. } => progress >= 1.0,
            Progress { progress, total, .. } => progress >= total,
            Task { done, .. } => done,
            Error { .. } => true,
        }
    }
    
    /// sets the widget such that it is done
    pub fn set_done(&mut self) {
        use WidgetType::*;
        match self.widget {
            None => {}
            Text { .. } => {}
            Percentage { ref mut progress, .. } => *progress = 1.0,
            Progress { ref mut progress, total, .. } => *progress = total,
            Task { ref mut done, .. } => *done = true,
            Error { .. } => {}
        }
    }

    /// Render the widget
    pub fn render(&self, time: time::SystemTime) {
        use WidgetType::*;

        const SPINNER: &'static str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";
        // change the first  half to change the "per second" spinner
        let factor = 1000 / 5;
        let charn = time
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .subsec_millis()
            / factor;
        let get_spinner_char = |cond| {
            if cond {
                "✓".to_string().green()
            } else {
                SPINNER
                    .chars()
                    .nth(charn as usize % SPINNER.len())
                    .unwrap()
                    .to_string()
                    .blue()
            }
        };

        match self.widget {
            None => println!(),
            // Lorem ipsum
            Text { message } => {
                println!("{message}")
            }
            // [⠦] [━━━━━━━━[ 55.0%]        ] Lorem ipsum
            Percentage { message, progress } => {
                let spinner_char = get_spinner_char(progress >= 1.0);
                println!(
                    "[{spinner_char}] [{center}] {message}",
                    center = percentage(progress, 25, format!("[{:3.1}%]", progress * 100.0))
                )
            }
            // [⠦] [━━━━━╸  [ 5/20]         ] Lorem ipsum
            Progress {
                message,
                progress,
                total,
            } => {
                let spinner_char = get_spinner_char(progress > total);
                let digits = (total as f32).log10().ceil() as usize;
                let percent_progress = progress as f32 / total as f32;
                println!(
                    "[{spinner_char}] [{center}] {message}",
                    center = percentage(
                        percent_progress,
                        25,
                        format!("[{progress:digits$}/{total}]")
                    )
                )
            }
            Task { message, done } => {
                let spinner_char = get_spinner_char(done);
                println!("[{spinner_char}] {message}")
            }
            // [⚠️] Uh-oh someone did an oopsie
            Error { message } => {
                println!("{}", format!("[⚠️] {message}").red())
            }
        }
    }
}

fn percentage(progress: f32, width: usize, center_msg: String) -> String {
    let center_width = center_msg.len();
    let left_width = (width - center_width) / 2;
    // the chunks of the progress bar that are filled
    let chunks = (progress * width as f32 * 2.0).floor() as usize;
    let tick = chunks % 2 == 1;
    let chunks = chunks / 2;

    let mut left = "━".repeat(left_width);
    let mut right = "".to_string();

    // if the progress is less than the left side of the center message
    if chunks < left_width {
        left = "━".repeat(chunks);
        if tick {
            left.push('╸');
        }
    }
    // if the progress is greater than the right side of the center message
    else if chunks >= left_width + center_width {
        right = "━".repeat(chunks - left_width - center_width);
        if tick {
            right.push('╸');
        }
    }

    let uncolored = format!(
        "{left:left_width$}{center_msg}{right:right_width$}",
        right_width = width - left_width - center_width
    );
    let left = uncolored.chars().take(chunks + 1).collect::<String>();
    let right = uncolored.chars().skip(chunks + 1).collect::<String>();
    format!("{}{}", left.green(), right)
}
