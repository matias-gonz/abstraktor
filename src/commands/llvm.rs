use std::path::{self, Path};

use clap::Parser;
use xshell::Shell;

use crate::logger::Logger;

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

pub fn run(args: LlvmArgs, logger: &Logger) {
    logger.log(format!("Instrumenting {}", args.path));
    let sh = Shell::new().unwrap();
    let instrumentor_path = path::absolute(Path::new(LLVM_INSTRUMENTOR_PATH)).unwrap();
    let path = Path::new(&args.path);
    let targets_path = path::absolute(Path::new(&args.targets_path)).unwrap();
    if !instrumentor_path.exists() {
        logger.error(format!(
            "instrumentor not found at {}",
            instrumentor_path.to_str().unwrap()
        ));
        return;
    }
    if !path.exists() {
        logger.error(format!("path not found at {}", path.to_str().unwrap()));
        return;
    }
    if !targets_path.exists() {
        logger.error(format!(
            "targets not found at {}",
            targets_path.to_str().unwrap()
        ));
        return;
    }

    sh.change_dir(path);
    sh.cmd("make")
        .envs([
            ("CC", instrumentor_path.to_str().unwrap()),
            ("TARGETS_FILE", targets_path.to_str().unwrap()),
            ("AFL_CC", AFL_CC),
        ])
        .run()
        .unwrap();
    logger.success("Instrumented binary");
}
