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
    let instrumentor_path = path::absolute(Path::new(&llvm_instrumentor_path))
        .context("Failed to absolutize instrumentor path")?;
    let path = Path::new(&args.path);
    let targets_path = path::absolute(Path::new(&args.targets_path))
        .context("Failed to absolutize targets path")?;
    if !instrumentor_path.exists() {
        logger.error(format!(
            "instrumentor not found at {}",
            instrumentor_path.to_str().unwrap()
        ));
        return Err(anyhow::anyhow!(
            "instrumentor not found at {}",
            instrumentor_path.to_str().unwrap()
        ));
    }
    if !path.exists() {
        logger.error(format!("path not found at {}", path.to_str().unwrap()));
        return Err(anyhow::anyhow!(
            "path not found at {}",
            path.to_str().unwrap()
        ));
    }
    if !targets_path.exists() {
        logger.error(format!(
            "targets not found at {}",
            targets_path.to_str().unwrap()
        ));
        return Err(anyhow::anyhow!(
            "targets not found at {}",
            targets_path.to_str().unwrap()
        ));
    }

    let _dir = sh.push_dir(path);
    let _cc = sh.push_env("CC", instrumentor_path.to_str().unwrap());
    let _targets = sh.push_env("TARGETS_FILE", targets_path.to_str().unwrap());
    let _afl_cc = sh.push_env("AFL_CC", AFL_CC);

    sh.cmd("sh")
        .arg("-c")
        .arg("./install.sh")
        .run()
        .context("Failed to run instrumentor")?;

    logger.success("Instrumented binary");
    Ok(())
}
