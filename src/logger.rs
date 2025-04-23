use std::fmt::Display;
use cliclack::{intro, log};
use console::style;

pub enum LogLevel {
    Debug,
    Info,
    Error,
}

pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }

    pub fn log(&self, message: impl Display) {
        log::info(message).unwrap();
    }

    pub fn success(&self, message: impl Display) {
        log::success(message).unwrap();
    }

    pub fn intro(&self) {
        intro(style(format!("Abstraktor - v{}", env!("CARGO_PKG_VERSION"))).bold().cyan()).unwrap();
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new(LogLevel::Info)
    }
}
