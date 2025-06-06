use std::path::Path;

use crate::{commands::GetTargetsArgs, logger::Logger};
use anyhow::{Context, Result};
use clap::Parser;

use super::{LlvmArgs, get_targets, llvm};

const TEMP_TARGETS_PATH: &str = "temp_targets.json";

#[derive(Parser, Debug)]
pub struct InstrumentArgs {
    #[arg(short, long)]
    path: String,
}

pub fn run(args: InstrumentArgs, logger: &Logger) -> Result<()> {
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
    };
    llvm::run(llvm_args, logger)?;
    std::fs::remove_file(temp_targets_path).context("Failed to remove temp targets file")?;
    Ok(())
}
