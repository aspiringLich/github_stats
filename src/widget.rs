use crossterm::style::{self, Stylize};

use std::{sync::RwLock, time};

pub enum Widget {
    // {message}
    Text {
        message: &'static str,
    },
    // [⠦] [━━━━━━━━[ 55.0%]        ] {message}
    Percentage {
        message: &'static str,
        progress: RwLock<f32>,
    },
    // [⠦] [━━━━━╸  [ 5/20]         ] {message}
    Progress {
        message: &'static str,
        progress: RwLock<usize>,
        total: usize,
    },
    // [⠦] {message}
    Task {
        message: &'static str,
        done: RwLock<bool>,
    },
    // [⚠️] {message}
    Error {
        message: &'static str,
    },
}

impl Widget {
    /// Create a new text widget
    pub fn new_text(message: &'static str) -> Self {
        Self::Text { message }
    }

    /// Create a new progress widget
    pub fn new_progress(message: &'static str) -> Self {
        Self::Percentage {
            message,
            progress: RwLock::new(0.0),
        }
    }

    /// Create a new discrete progress widget
    pub fn new_discrete_progress(message: &'static str, total: usize) -> Self {
        Self::Progress {
            message,
            progress: RwLock::new(0),
            total,
        }
    }

    /// Create a new error widget
    pub fn new_error(message: &'static str) -> Self {
        Self::Error { message }
    }

    /// Update the progress of a progress widget
    pub fn update_progress(&self, progress: f32) {
        if let Self::Percentage { progress: p, .. } = self {
            *p.write().unwrap() = progress;
        }
    }

    /// Update the progress of a discrete progress widget
    pub fn update_discrete_progress(&self, progress: usize) {
        if let Self::Progress { progress: p, .. } = self {
            *p.write().unwrap() = progress;
        }
    }
    
    /// Update whether a task is done
    pub fn update_task_done(&self, done: bool) {
        if let Self::Task { done: d, .. } = self {
            *d.write().unwrap() = done;
        }
    }

    /// Render the widget
    pub fn render(&self, time: time::SystemTime) {
        use Widget::*;

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
        }};

        match self {
            // Lorem ipsum
            Text { message } => {
                println!("{message}")
            }
            // [⠦] [━━━━━━━━[ 55.0%]        ] Lorem ipsum
            Percentage { message, progress } => {
                let progress = *progress.read().unwrap();
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
                let progress = *progress.read().unwrap();
                let spinner_char = get_spinner_char(progress > *total);
                let digits = (*total as f32).log10().ceil() as usize;
                let percent_progress = progress as f32 / *total as f32;
                println!(
                    "[{spinner_char}] [{center}] {message}",
                    center = percentage(
                        percent_progress,
                        25,
                        format!("[{progress:digits$}/{total}]")
                    )
                )
            }
            Task {
                message,
                done,
            } => {
                let done = *done.read().unwrap();
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
