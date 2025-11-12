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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new(LogLevel::Debug);
        assert_eq!(logger.level, LogLevel::Debug);

        let logger = Logger::new(LogLevel::Info);
        assert_eq!(logger.level, LogLevel::Info);

        let logger = Logger::new(LogLevel::Error);
        assert_eq!(logger.level, LogLevel::Error);

        let logger = Logger::new(LogLevel::Quiet);
        assert_eq!(logger.level, LogLevel::Quiet);
    }

    #[test]
    fn test_logger_default() {
        let logger = Logger::default();
        assert_eq!(logger.level, LogLevel::Info);
    }

    #[test]
    fn test_debug_level_shows_all() {
        let logger = Logger::new(LogLevel::Debug);
        logger.debug("test debug");
        logger.log("test log");
        logger.success("test success");
        logger.warning("test warning");
        logger.intro();
        logger.outro();
        logger.error("test error");
    }

    #[test]
    fn test_info_level_hides_debug() {
        let logger = Logger::new(LogLevel::Info);
        logger.debug("test debug");
        logger.log("test log");
        logger.success("test success");
        logger.warning("test warning");
        logger.intro();
        logger.outro();
        logger.error("test error");
    }

    #[test]
    fn test_error_level_hides_info() {
        let logger = Logger::new(LogLevel::Error);
        logger.debug("test debug");
        logger.log("test log");
        logger.success("test success");
        logger.warning("test warning");
        logger.intro();
        logger.outro();
        logger.error("test error");
    }

    #[test]
    fn test_quiet_level_hides_all() {
        let logger = Logger::new(LogLevel::Quiet);
        logger.debug("test debug");
        logger.log("test log");
        logger.success("test success");
        logger.warning("test warning");
        logger.intro();
        logger.outro();
        logger.error("test error");
    }

    #[test]
    fn test_log_level_variants() {
        assert_eq!(LogLevel::Debug, LogLevel::Debug);
        assert_ne!(LogLevel::Debug, LogLevel::Info);
        assert_ne!(LogLevel::Debug, LogLevel::Error);
        assert_ne!(LogLevel::Debug, LogLevel::Quiet);
    }

    #[test]
    fn test_display_trait_with_strings() {
        let logger = Logger::new(LogLevel::Debug);
        logger.debug("string literal");
        logger.log(String::from("owned string"));
        logger.success(format!("formatted {}", "string"));
        logger.warning(&"string reference");
    }

    #[test]
    fn test_display_trait_with_numbers() {
        let logger = Logger::new(LogLevel::Debug);
        logger.debug(42);
        logger.log(3.14);
        logger.success(100u64);
    }
}
