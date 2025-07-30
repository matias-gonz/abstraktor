use clap::{Parser, Subcommand};
use anyhow::Result;
use crate::logger::Logger;

pub mod docker;
pub use docker::DockerArgs;

#[derive(Parser, Debug)]
pub struct SetupArgs {
    #[command(subcommand)]
    pub command: SetupSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum SetupSubcommand {
    Docker(DockerArgs),
}

pub fn run(args: SetupArgs, logger: &Logger) -> Result<()> {
    match args.command {
        SetupSubcommand::Docker(args) => docker::run(args, logger)?,
    }
    Ok(())
} 