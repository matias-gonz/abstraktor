use std::path::{self, Path};

use crate::logger::Logger;
use anyhow::{Context, Result};
use clap::Parser;
use xshell::Shell;

const LLVM_INSTRUMENTOR_PATH: &str = "./llvm/afl-clang-fast";

const AFL_CC: &str = "clang-12";
const AFL_QUIET: &str = "1";

#[derive(Parser, Debug)]
pub struct LlvmArgs {
    #[arg(short, long)]
    pub path: String,
    #[arg(short, long)]
    pub targets_path: String,
}

pub fn run(args: LlvmArgs, logger: &Logger) -> Result<()> {
    logger.log(format!("Instrumenting {}", args.path));
    let sh = Shell::new()?;
    let instrumentor_path = path::absolute(Path::new(LLVM_INSTRUMENTOR_PATH))
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
    sh.change_dir(path);
    sh.cmd("sh")
        .args(["-c", "./install.sh"])
        .envs([
            ("CC", instrumentor_path.to_str().unwrap()),
            ("TARGETS_FILE", targets_path.to_str().unwrap()),
            ("AFL_CC", AFL_CC),
        ])
        .run()
        .context("Failed to run instrumentor")?;
    logger.success("Instrumented binary");
    Ok(())
}
