mod commands;
mod logger;
mod model;
use anyhow::Result;
use clap::Parser;
use commands::{Abstraktor, AbstraktorSubcommand};
use logger::Logger;
use xshell::Shell;

fn main() -> Result<()> {
    let args = Abstraktor::parse();
    let logger = Logger::default();
    let sh = Shell::new()?;
    logger.intro();
    match args.command {
        AbstraktorSubcommand::GetTargets(args) => commands::get_targets::run(args, &logger)?,
        AbstraktorSubcommand::Llvm(args) => commands::llvm::run(args, &logger, &sh)?,
        AbstraktorSubcommand::Instrument(args) => commands::instrument::run(args, &logger, &sh)?,
        AbstraktorSubcommand::Setup(args) => commands::setup::run(args, &logger, &sh)?,
        AbstraktorSubcommand::Run(args) => commands::run::run(args, &logger, &sh)?,
        AbstraktorSubcommand::Clean(args) => commands::clean::run(args, &logger, &sh)?,
        AbstraktorSubcommand::ExportGraphs(args) => commands::export_graphs::run(args, &logger)?,
    };
    logger.outro();
    Ok(())
}
