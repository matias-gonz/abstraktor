use crate::logger::Logger;
use anyhow::Result;
use clap::{Parser, Subcommand};
use xshell::Shell;

pub mod mallory;
pub use mallory::RunMalloryArgs;

pub mod mediator;
pub use mediator::RunMediatorArgs;

#[derive(Parser, Debug)]
pub struct RunArgs {
    #[command(subcommand)]
    pub command: RunSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum RunSubcommand {
    Mallory(RunMalloryArgs),
    Mediator(RunMediatorArgs),
}

pub fn run(args: RunArgs, logger: &Logger, sh: &Shell) -> Result<()> {
    match args.command {
        RunSubcommand::Mallory(args) => mallory::run(args, logger, sh)?,
        RunSubcommand::Mediator(args) => mediator::run(args, logger, sh)?,
    }
    Ok(())
}
