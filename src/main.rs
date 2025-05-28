mod commands;
mod logger;
mod model;
use anyhow::Result;
use clap::Parser;
use commands::{Abstraktor, AbstraktorSubcommand};
use logger::Logger;

fn main() -> Result<()> {
    let args = Abstraktor::parse();
    let logger = Logger::default();
    logger.intro();
    match args.command {
        AbstraktorSubcommand::GetTargets(args) => commands::get_targets::run(args, &logger)?,
        AbstraktorSubcommand::Llvm(args) => commands::llvm::run(args, &logger)?,
        AbstraktorSubcommand::Instrument(args) => commands::instrument::run(args, &logger)?,
    };
    logger.outro();
    Ok(())
}
