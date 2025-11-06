use clap::ValueEnum;
use cliclack::{intro, log, outro};
use console::style;
use std::fmt::Display;

#[derive(PartialEq, Clone, Copy, Debug, ValueEnum)]
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

    pub fn debug(&self, message: impl Display) {
        if self.level != LogLevel::Debug {
            return;
        }

        log::step(format!("{} {}", style("[DEBUG]").dim(), message)).unwrap();
    }

    pub fn log(&self, message: impl Display) {
        if self.level == LogLevel::Quiet || self.level == LogLevel::Error {
            return;
        }

        log::step(message).unwrap();
    }

    pub fn success(&self, message: impl Display) {
        if self.level == LogLevel::Quiet || self.level == LogLevel::Error {
            return;
        }

        log::success(message).unwrap();
    }

    pub fn warning(&self, message: impl Display) {
        if self.level == LogLevel::Quiet || self.level == LogLevel::Error {
            return;
        }

        log::warning(format!(
            "{} {}",
            style("Warning:").bold().yellow(),
            style(message).yellow()
        ))
        .unwrap();
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
