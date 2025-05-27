use cliclack::{intro, log, outro};
use console::style;
use std::fmt::Display;

#[derive(PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Error,
    Quiet,
}

pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }

    pub fn log(&self, message: impl Display) {
        if self.level == LogLevel::Quiet {
            return;
        }

        log::step(message).unwrap();
    }

    pub fn success(&self, message: impl Display) {
        if self.level == LogLevel::Quiet {
            return;
        }

        log::success(message).unwrap();
    }

    pub fn intro(&self) {
        if self.level == LogLevel::Quiet {
            return;
        }

        intro(
            style(format!("Abstraktor - v{}", env!("CARGO_PKG_VERSION")))
                .bold()
                .cyan(),
        )
        .unwrap();
    }

    pub fn outro(&self) {
        if self.level == LogLevel::Quiet {
            return;
        }

        outro(style("Execution successful!").bold().green()).unwrap();
    }

    pub fn error(&self, message: impl Display) {
        if self.level == LogLevel::Quiet {
            return;
        }

        log::error(format!(
            "{} {}",
            style("Error:").bold().red(),
            style(message).red()
        ))
        .unwrap();
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new(LogLevel::Info)
    }
}
