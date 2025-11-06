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

    let llvm_instrumentor_path = args.llvm_path.unwrap_or_else(|| LLVM_INSTRUMENTOR_PATH.to_string());
    logger.debug(format!("Using LLVM instrumentor at: {}", llvm_instrumentor_path));
    
    let instrumentor_path = path::absolute(Path::new(&llvm_instrumentor_path))
        .context("Failed to absolutize instrumentor path")?;
    let path = Path::new(&args.path);
    let targets_path = path::absolute(Path::new(&args.targets_path))
        .context("Failed to absolutize targets path")?;
    
    logger.debug(format!("Instrumentor path: {}", instrumentor_path.display()));
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
