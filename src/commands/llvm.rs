use std::path::Path;

use clap::Parser;
use xshell::Shell;

const LLVM_INSTRUMENTOR_PATH: &str = "../llvm/afl-clang-fast";

#[derive(Parser, Debug)]
pub struct LlvmArgs {
    #[arg(short, long)]
    path: String,
    #[arg(short, long)]
    targets_path: String,
    #[arg(short, long)]
    output: String,
}

pub fn run(_args: LlvmArgs) {
    let sh = Shell::new().unwrap();
    let instrumentor_path = Path::new(LLVM_INSTRUMENTOR_PATH);
    if !instrumentor_path.exists() {
        println!("Error: instrumentor not found");
        return;
    }
}

