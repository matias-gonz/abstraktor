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
    let temp_targets_path = Path::new(TEMP_TARGETS_PATH);
    let temp_targets_path_str = temp_targets_path
        .to_str()
        .context("Failed to get temp targets path")?
        .to_string();
    let get_targets_args = GetTargetsArgs {
        path: args.path.clone(),
        output: temp_targets_path_str.clone(),
    };

    get_targets::run(get_targets_args, logger)?;
    let llvm_args = LlvmArgs {
        path: args.path.clone(),
        targets_path: temp_targets_path_str.clone(),
        llvm_path: args.llvm_path,
    };
    llvm::run(llvm_args, logger, sh)?;
    std::fs::remove_file(temp_targets_path).context("Failed to remove temp targets file")?;
    Ok(())
}
