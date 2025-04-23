use std::path::Path;

use clap::Parser;
use xshell::Shell;

use crate::logger::Logger;

const LLVM_INSTRUMENTOR_PATH: &str = "../llvm/afl-clang-fast";

const AFL_CC: &str = "clang-12";
const AFL_QUIET: &str = "1";

#[derive(Parser, Debug)]
pub struct LlvmArgs {
    #[arg(short, long)]
    pub path: String,
    #[arg(short, long)]
    pub targets_path: String,
    #[arg(short, long)]
    pub output: String,
}

pub fn run(args: LlvmArgs, logger: &Logger) {
    logger.log(format!("Instrumenting {}", args.path));
    let sh = Shell::new().unwrap();
    let instrumentor_path = Path::new(LLVM_INSTRUMENTOR_PATH);
    let path = Path::new(&args.path);
    let targets_path = Path::new(&args.targets_path);
    let output_path = Path::new(&args.output);
    if !instrumentor_path.exists() {
        logger.error(format!("instrumentor not found at {}", instrumentor_path.to_str().unwrap()));
        return;
    }
    sh.cmd(instrumentor_path)
    .args(&["-o", output_path.to_str().unwrap(), path.to_str().unwrap()])
    .envs([
        ("TARGETS_FILE", targets_path.to_str().unwrap()),
        ("AFL_QUIET", AFL_QUIET),
        ("AFL_CC", AFL_CC)
        ])
        .quiet()
        .run()
        .unwrap();
    logger.success(format!("Instrumented binary saved to {}", args.output));
}

