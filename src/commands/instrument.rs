use std::path::Path;

use clap::Parser;

use crate::{commands::GetTargetsArgs, logger::Logger};

use super::{get_targets, llvm, LlvmArgs};

const TEMP_TARGETS_PATH: &str = "temp_targets.json";


#[derive(Parser, Debug)]
pub struct InstrumentArgs {
    #[arg(short, long)]
    path: String,
    #[arg(short, long)]
    output: String,
}

pub fn run(args: InstrumentArgs, logger: &Logger) {
    let temp_targets_path = Path::new(TEMP_TARGETS_PATH);
    let get_targets_args = GetTargetsArgs {
        path: args.path.clone(),
        output: temp_targets_path.to_str().unwrap().to_string(),
    };
    get_targets::run(get_targets_args, logger);
    let llvm_args = LlvmArgs {
        path: args.path,
        targets_path: temp_targets_path.to_str().unwrap().to_string(),
        output: args.output,
    };
    llvm::run(llvm_args, logger);
    std::fs::remove_file(temp_targets_path).unwrap();
}
