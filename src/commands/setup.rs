use clap::{Parser, Subcommand};
use anyhow::Result;
use crate::logger::Logger;

pub mod docker;
pub use docker::DockerArgs;

pub mod mediator;
pub use mediator::MediatorArgs;

#[derive(Parser, Debug)]
pub struct SetupArgs {
    #[command(subcommand)]
    pub command: SetupSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum SetupSubcommand {
    Docker(DockerArgs),
    Mediator(MediatorArgs),
}

pub fn run(args: SetupArgs, logger: &Logger) -> Result<()> {
    match args.command {
        SetupSubcommand::Docker(args) => docker::run(args, logger)?,
        SetupSubcommand::Mediator(args) => mediator::run(args, logger)?,
    }
    Ok(())
} 