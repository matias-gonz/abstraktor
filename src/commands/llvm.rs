use std::path::{self, Path};

use crate::logger::Logger;
use anyhow::{Context, Result};
use clap::Parser;
use xshell::Shell;

const LLVM_INSTRUMENTOR_PATH: &str = "./llvm/afl-clang-fast";

const AFL_CC: &str = "clang-11";

#[derive(Parser, Debug)]
pub struct LlvmArgs {
    #[arg(short, long)]
    pub path: String,
    #[arg(short, long)]
    pub targets_path: String,
    #[arg(short, long)]
    pub llvm_path: Option<String>,
}

pub fn run(args: LlvmArgs, logger: &Logger, sh: &Shell) -> Result<()> {
    logger.log(format!("Instrumenting {}", args.path));

    let llvm_instrumentor_path = args
        .llvm_path
        .unwrap_or_else(|| LLVM_INSTRUMENTOR_PATH.to_string());
    logger.debug(format!(
        "Using LLVM instrumentor at: {}",
        llvm_instrumentor_path
    ));

    let instrumentor_path = path::absolute(Path::new(&llvm_instrumentor_path))
        .context("Failed to absolutize instrumentor path")?;
    let path = Path::new(&args.path);
    let targets_path = path::absolute(Path::new(&args.targets_path))
        .context("Failed to absolutize targets path")?;

    logger.debug(format!(
        "Instrumentor path: {}",
        instrumentor_path.display()
    ));
    logger.debug(format!("Source path: {}", path.display()));
    logger.debug(format!("Targets file: {}", targets_path.display()));

    if !instrumentor_path.exists() {
        logger.error(format!(
            "Instrumentor not found at {}",
            instrumentor_path.to_str().unwrap()
        ));
        return Err(anyhow::anyhow!(
            "Instrumentor not found at {}",
            instrumentor_path.to_str().unwrap()
        ));
    }
    if !path.exists() {
        logger.error(format!("Path not found at {}", path.to_str().unwrap()));
        return Err(anyhow::anyhow!(
            "Path not found at {}",
            path.to_str().unwrap()
        ));
    }
    if !targets_path.exists() {
        logger.error(format!(
            "Targets file not found at {}",
            targets_path.to_str().unwrap()
        ));
        return Err(anyhow::anyhow!(
            "Targets file not found at {}",
            targets_path.to_str().unwrap()
        ));
    }

    logger.log("Setting up environment variables for instrumentation");
    let _dir = sh.push_dir(path);
    let _cc = sh.push_env("CC", instrumentor_path.to_str().unwrap());
    let _targets = sh.push_env("TARGETS_FILE", targets_path.to_str().unwrap());
    let _afl_cc = sh.push_env("AFL_CC", AFL_CC);

    logger.debug(format!("CC={}", instrumentor_path.display()));
    logger.debug(format!("TARGETS_FILE={}", targets_path.display()));
    logger.debug(format!("AFL_CC={}", AFL_CC));

    logger.log("Running install.sh script");
    sh.cmd("sh")
        .arg("-c")
        .arg("./install.sh")
        .run()
        .context("Failed to run instrumentor")?;

    logger.success("Binary instrumented successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::{LogLevel, Logger};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_llvm_args_creation() {
        let args = LlvmArgs {
            path: "/test/path".to_string(),
            targets_path: "/test/targets.json".to_string(),
            llvm_path: Some("/custom/llvm".to_string()),
        };
        assert_eq!(args.path, "/test/path");
        assert_eq!(args.targets_path, "/test/targets.json");
        assert_eq!(args.llvm_path, Some("/custom/llvm".to_string()));
    }

    #[test]
    fn test_llvm_args_without_custom_path() {
        let args = LlvmArgs {
            path: "/test/path".to_string(),
            targets_path: "/test/targets.json".to_string(),
            llvm_path: None,
        };
        assert_eq!(args.path, "/test/path");
        assert_eq!(args.targets_path, "/test/targets.json");
        assert!(args.llvm_path.is_none());
    }

    #[test]
    fn test_llvm_instrumentor_path_constant() {
        assert_eq!(LLVM_INSTRUMENTOR_PATH, "./llvm/afl-clang-fast");
    }

    #[test]
    fn test_afl_cc_constant() {
        assert_eq!(AFL_CC, "clang-11");
    }

    #[test]
    fn test_llvm_path_defaults_to_constant() {
        let args = LlvmArgs {
            path: "./test".to_string(),
            targets_path: "./targets.json".to_string(),
            llvm_path: None,
        };

        let llvm_path = args
            .llvm_path
            .unwrap_or_else(|| LLVM_INSTRUMENTOR_PATH.to_string());
        assert_eq!(llvm_path, LLVM_INSTRUMENTOR_PATH);
    }

    #[test]
    fn test_llvm_path_uses_custom_when_provided() {
        let custom_path = "/custom/instrumentor";
        let args = LlvmArgs {
            path: "./test".to_string(),
            targets_path: "./targets.json".to_string(),
            llvm_path: Some(custom_path.to_string()),
        };

        let llvm_path = args
            .llvm_path
            .unwrap_or_else(|| LLVM_INSTRUMENTOR_PATH.to_string());
        assert_eq!(llvm_path, custom_path);
    }

    #[test]
    fn test_run_with_nonexistent_instrumentor() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test_dir");
        fs::create_dir(&test_path).unwrap();

        let targets_file = temp_dir.path().join("targets.json");
        fs::write(&targets_file, "[]").unwrap();

        let logger = Logger::new(LogLevel::Quiet);
        let sh = Shell::new().unwrap();

        let args = LlvmArgs {
            path: test_path.to_string_lossy().to_string(),
            targets_path: targets_file.to_string_lossy().to_string(),
            llvm_path: Some("/nonexistent/instrumentor/path".to_string()),
        };

        let result = run(args, &logger, &sh);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Instrumentor not found")
        );
    }

    #[test]
    fn test_run_with_nonexistent_path() {
        let temp_dir = TempDir::new().unwrap();
        let targets_file = temp_dir.path().join("targets.json");
        fs::write(&targets_file, "[]").unwrap();

        let fake_instrumentor = temp_dir.path().join("fake_instrumentor");
        fs::write(&fake_instrumentor, "#!/bin/sh\necho test").unwrap();

        let logger = Logger::new(LogLevel::Quiet);
        let sh = Shell::new().unwrap();

        let args = LlvmArgs {
            path: "/nonexistent/source/path".to_string(),
            targets_path: targets_file.to_string_lossy().to_string(),
            llvm_path: Some(fake_instrumentor.to_string_lossy().to_string()),
        };

        let result = run(args, &logger, &sh);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Path not found"));
    }

    #[test]
    fn test_run_with_nonexistent_targets_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test_dir");
        fs::create_dir(&test_path).unwrap();

        let fake_instrumentor = temp_dir.path().join("fake_instrumentor");
        fs::write(&fake_instrumentor, "#!/bin/sh\necho test").unwrap();

        let logger = Logger::new(LogLevel::Quiet);
        let sh = Shell::new().unwrap();

        let args = LlvmArgs {
            path: test_path.to_string_lossy().to_string(),
            targets_path: "/nonexistent/targets.json".to_string(),
            llvm_path: Some(fake_instrumentor.to_string_lossy().to_string()),
        };

        let result = run(args, &logger, &sh);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Targets file not found")
        );
    }

    #[test]
    fn test_args_with_various_path_formats() {
        let test_cases = vec![
            ("./relative/path", "./targets.json"),
            ("/absolute/path", "/absolute/targets.json"),
            ("simple_path", "targets.json"),
            ("../parent/path", "../targets.json"),
        ];

        for (path, targets_path) in test_cases {
            let args = LlvmArgs {
                path: path.to_string(),
                targets_path: targets_path.to_string(),
                llvm_path: None,
            };
            assert_eq!(args.path, path);
            assert_eq!(args.targets_path, targets_path);
        }
    }

    #[test]
    fn test_validation_order_checks_instrumentor_first() {
        let logger = Logger::new(LogLevel::Quiet);
        let sh = Shell::new().unwrap();

        let args = LlvmArgs {
            path: "/nonexistent1".to_string(),
            targets_path: "/nonexistent2".to_string(),
            llvm_path: Some("/nonexistent_instrumentor".to_string()),
        };

        let result = run(args, &logger, &sh);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Instrumentor not found"));
    }
}
