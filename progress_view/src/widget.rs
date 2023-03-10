use crossterm::style::Stylize;

use std::{
    default::default,
    time,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub enum WidgetType {
    // {message}
    #[default]
    Text,
    // [⠦] [━━━━━━━━[ 55.0%]        ] {message}
    Percentage {
        progress: f32,
    },
    // [⠦] [━━━━━╸  [ 5/20]         ] {message}
    Progress {
        progress: usize,
        total: usize,
    },
    // [⠦] {message}
    Task {
        done: bool,
    },
    // [⚠️] {message}
    Error,
}

#[derive(Default, Debug, PartialEq)]
pub struct Widget {
    pub widget: WidgetType,
    pub active: bool,
    pub message: String,
    pub indent: usize,
}

impl Widget {
    /// Creates a new text widget off of a WidgetType, message, and indent
    fn new<T: Into<String>>(message: T, indent: usize, widget: WidgetType) -> Self {
        Self {
            message: message.into(),
            indent,
            widget,
            ..default()
        }
    }

    /// Create a new text widget
    pub fn new_text<T: Into<String>>(message: T, indent: usize) -> Self {
        Self::new(message, indent, WidgetType::Text)
    }

    /// Create a new progress widget
    pub fn new_progress<T: Into<String>>(message: T, indent: usize, total: usize) -> Self {
        Self::new(message, indent, WidgetType::Progress { progress: 0, total })
    }

    /// Create a new discrete progress widget
    pub fn new_percentage<T: Into<String>>(message: T, indent: usize) -> Self {
        Self::new(message, indent, WidgetType::Percentage { progress: 0.0 })
    }

    /// Create a new task widget
    pub fn new_task<T: Into<String>>(message: T, indent: usize) -> Self {
        Self::new(message, indent, WidgetType::Task { done: false })
    }

    /// Create a new error widget
    pub fn new_error<T: Into<String>>(message: T, indent: usize) -> Self {
        Self::new(message, indent, WidgetType::Error)
    }

    /// Update the progress of a progress widget
    pub fn update_progress(&mut self, progress: f32) {
        if let WidgetType::Percentage {
            progress: ref mut p,
            ..
        } = self.widget
        {
            self.active = true;
            *p = progress;
        }
    }

    /// Update the progress of a discrete progress widget
    pub fn update_discrete_progress(&mut self, progress: usize) {
        if let WidgetType::Progress {
            progress: ref mut p,
            ..
        } = self.widget
        {
            self.active = true;
            *p = progress;
        }
    }

    /// Update whether a task is done
    pub fn update_task_done(&mut self, done: bool) {
        if let WidgetType::Task {
            done: ref mut d, ..
        } = self.widget
        {
            self.active = true;
            *d = done;
        }
    }

    /// detects if this widget is done
    pub fn is_done(&self) -> bool {
        use WidgetType::*;
        match self.widget {
            Text { .. } => true,
            Percentage { progress, .. } => progress >= 1.0,
            Progress {
                progress, total, ..
            } => progress >= total,
            Task { done, .. } => done,
            Error { .. } => true,
        }
    }

    /// detects if this widget is active
    /// widgets that dont track progress are automatically active
    pub fn is_active(&self) -> bool {
        use WidgetType::*;
        match self.widget {
            Text { .. } => true,
            Percentage { .. } => self.active,
            Progress { .. } => self.active,
            Task { .. } => self.active,
            Error { .. } => true,
        }
    }

    /// sets the widget such that it is done
    pub fn set_done(&mut self) {
        use WidgetType::*;
        self.active = true;
        match self.widget {
            Text { .. } => {}
            Percentage {
                ref mut progress, ..
            } => *progress = 1.0,
            Progress {
                ref mut progress,
                total,
                ..
            } => *progress = total,
            Task { ref mut done, .. } => *done = true,
            Error { .. } => {}
        }
        // dbg!(&self);
    }

    /// sets the message of this widget
    pub fn set_message<T: Into<String>>(&mut self, message: T) {
        self.message = message.into();
    }

    /// Render the widget
    pub fn render(&self, time: time::SystemTime) {
        use WidgetType::*;

        const SPINNER: [char; 6] = ['⠋', '⠙', '⠸', '⣠', '⣄', '⡆'];
        let charn = (time.duration_since(time::UNIX_EPOCH).unwrap().as_millis() / 100
            % usize::MAX as u128) as usize;
        let get_spinner_char = |cond| {
            if self.active {
                if cond {
                    "✓".to_string().green()
                } else {
                    SPINNER[charn % SPINNER.len()].to_string().white()
                }
            } else {
                " ".to_string().white()
            }
        };

        let message = &self.message;
        match &self.widget {
            // Lorem ipsum
            Text => {
                println!("{message}")
            }
            // [⠦] [━━━━━━━━[ 55.0%]        ] Lorem ipsum
            Percentage { progress } => {
                let spinner_char = get_spinner_char(*progress >= 1.0);
                println!(
                    "[{spinner_char}] [{center}] {message}",
                    center = percentage(*progress, 25, format!("[{:3.1}%]", *progress * 100.0))
                )
            }
            // [⠦] [━━━━━╸  [ 5/20]         ] Lorem ipsum
            Progress { progress, total } => {
                let spinner_char = get_spinner_char(*progress > *total);
                let digits = (*total as f32).log10().ceil() as usize;
                let percent_progress = *progress as f32 / *total as f32;
                println!(
                    "[{spinner_char}] [{center}] {message}",
                    center = percentage(
                        percent_progress,
                        25,
                        format!("[{progress:digits$}/{total}]")
                    )
                )
            }
            Task { done } => {
                let spinner_char = get_spinner_char(*done);
                println!("[{spinner_char}] {message}")
            }
            // [⚠️] Uh-oh someone did an oopsie
            Error => {
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
