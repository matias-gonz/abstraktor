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
		AbstraktorSubcommand::RunMallory(args) => commands::run_mallory::run(args, &logger, &sh)?,
		AbstraktorSubcommand::RunMediator(args) => commands::run_mediator::run(args, &logger, &sh)?,
	};
	logger.outro();
	Ok(())
}
