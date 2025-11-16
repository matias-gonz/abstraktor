use std::path::Path;

use crate::{commands::GetTargetsArgs, logger::Logger};
use anyhow::{Context, Result};
use clap::Parser;
use xshell::Shell;

use super::{LlvmArgs, get_targets, llvm};

const TEMP_TARGETS_PATH: &str = "temp_targets.json";

#[derive(Parser, Debug)]
pub struct InstrumentArgs {
    #[arg(short, long)]
    path: String,

    #[arg(short, long)]
    llvm_path: Option<String>,
}

pub fn run(args: InstrumentArgs, logger: &Logger, sh: &Shell) -> Result<()> {
    run_with_temp_path(args, logger, sh, TEMP_TARGETS_PATH)
}

fn run_with_temp_path(
    args: InstrumentArgs,
    logger: &Logger,
    sh: &Shell,
    temp_path: &str,
) -> Result<()> {
    logger.log("Starting instrumentation process");
    logger.debug(format!("Target path: {}", args.path));

    let temp_targets_path = Path::new(temp_path);
    let temp_targets_path_str = temp_targets_path
        .to_str()
        .context("Failed to get temp targets path")?
        .to_string();
    logger.debug(format!(
        "Using temporary targets file: {}",
        temp_targets_path_str
    ));

    let get_targets_args = GetTargetsArgs {
        path: args.path.clone(),
        output: temp_targets_path_str.clone(),
    };

    logger.log("Step 1/2: Analyzing source code for targets");
    get_targets::run(get_targets_args, logger)?;

    logger.log("Step 2/2: Instrumenting binary with LLVM");
    let llvm_args = LlvmArgs {
        path: args.path.clone(),
        targets_path: temp_targets_path_str.clone(),
        llvm_path: args.llvm_path,
    };
    llvm::run(llvm_args, logger, sh)?;

    logger.debug("Cleaning up temporary targets file");
    std::fs::remove_file(temp_targets_path).context("Failed to remove temp targets file")?;

    logger.success("Instrumentation completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::{LogLevel, Logger};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_instrument_args_creation() {
        let args = InstrumentArgs {
            path: "/test/path".to_string(),
            llvm_path: Some("/custom/llvm".to_string()),
        };
        assert_eq!(args.path, "/test/path");
        assert_eq!(args.llvm_path, Some("/custom/llvm".to_string()));
    }

    #[test]
    fn test_instrument_args_without_llvm_path() {
        let args = InstrumentArgs {
            path: "/test/path".to_string(),
            llvm_path: None,
        };
        assert_eq!(args.path, "/test/path");
        assert!(args.llvm_path.is_none());
    }

    #[test]
    fn test_get_targets_step() {
        let test_dir = Path::new("./tests/instrument_test");
        if !test_dir.exists() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let temp_targets = temp_dir.path().join("test_targets.json");

        let logger = Logger::new(LogLevel::Quiet);
        let get_targets_args = GetTargetsArgs {
            path: test_dir.to_string_lossy().into_owned(),
            output: temp_targets.to_string_lossy().into_owned(),
        };

        let result = get_targets::run(get_targets_args, &logger);
        assert!(result.is_ok());
        assert!(temp_targets.exists());

        let content = fs::read_to_string(&temp_targets).unwrap();
        let targets: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(targets.is_array());

        let array = targets.as_array().unwrap();
        assert!(!array.is_empty());
    }

    #[test]
    fn test_temp_targets_path_constant() {
        assert_eq!(TEMP_TARGETS_PATH, "temp_targets.json");
    }

    #[test]
    fn test_run_with_temp_path_fails_with_bad_instrumentor() {
        let test_dir = Path::new("./tests/instrument_test");
        if !test_dir.exists() {
            return;
        }

        let logger = Logger::new(LogLevel::Quiet);
        let sh = Shell::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let temp_targets = temp_dir.path().join("temp_test_targets.json");

        let args = InstrumentArgs {
            path: test_dir.to_string_lossy().into_owned(),
            llvm_path: Some("/fake/llvm/path".to_string()),
        };

        let result =
            run_with_temp_path(args, &logger, &sh, temp_targets.to_string_lossy().as_ref());

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Instrumentor not found"));

        if temp_targets.exists() {
            fs::remove_file(&temp_targets).unwrap();
        }
    }

    #[test]
    fn test_run_with_temp_path_generates_targets() {
        let test_dir = Path::new("./tests/instrument_test");
        if !test_dir.exists() {
            return;
        }

        let logger = Logger::new(LogLevel::Quiet);
        let temp_dir = TempDir::new().unwrap();
        let temp_targets = temp_dir.path().join("generate_test_targets.json");

        let get_targets_args = GetTargetsArgs {
            path: test_dir.to_string_lossy().into_owned(),
            output: temp_targets.to_string_lossy().into_owned(),
        };

        let result = get_targets::run(get_targets_args, &logger);
        assert!(result.is_ok());
        assert!(temp_targets.exists());

        let content = fs::read_to_string(&temp_targets).unwrap();
        let targets: serde_json::Value = serde_json::from_str(&content).unwrap();

        let array = targets.as_array().unwrap();
        assert_eq!(array.len(), 2);
    }

    #[test]
    fn test_run_with_temp_path_step_1_creates_targets_file() {
        let test_dir = Path::new("./tests/instrument_test");
        if !test_dir.exists() {
            return;
        }

        let logger = Logger::new(LogLevel::Quiet);
        let temp_dir = TempDir::new().unwrap();
        let temp_targets = temp_dir.path().join("step1_test_targets.json");

        let get_targets_args = GetTargetsArgs {
            path: test_dir.to_string_lossy().into_owned(),
            output: temp_targets.to_string_lossy().into_owned(),
        };

        get_targets::run(get_targets_args, &logger).unwrap();
        assert!(temp_targets.exists());

        let content = fs::read_to_string(&temp_targets).unwrap();
        assert!(content.contains("main.c"));
        assert!(content.contains("math_utils.c"));
    }

    #[test]
    fn test_instrument_args_path_handling() {
        let test_cases = vec![
            "./relative/path",
            "/absolute/path",
            "simple_path",
            "../parent/path",
        ];

        for path in test_cases {
            let args = InstrumentArgs {
                path: path.to_string(),
                llvm_path: None,
            };
            assert_eq!(args.path, path);
        }
    }

    #[test]
    fn test_llvm_path_options() {
        let with_custom = InstrumentArgs {
            path: "./test".to_string(),
            llvm_path: Some("/custom/llvm".to_string()),
        };
        assert!(with_custom.llvm_path.is_some());

        let with_default = InstrumentArgs {
            path: "./test".to_string(),
            llvm_path: None,
        };
        assert!(with_default.llvm_path.is_none());
    }
}
