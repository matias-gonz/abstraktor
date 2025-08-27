use crate::logger::Logger;
use anyhow::Result;
use clap::{Parser, Subcommand};
use xshell::Shell;

pub mod docker;
pub use docker::DockerArgs;

pub mod mediator;
pub use mediator::MediatorArgs;

pub mod all;
pub use all::SetupAllArgs;

pub mod sut;
pub use sut::SutArgs;

pub mod sut_common;

pub mod jepsen;
pub use jepsen::JepsenArgs;

#[derive(Parser, Debug)]
pub struct SetupArgs {
    #[command(subcommand)]
    pub command: SetupSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum SetupSubcommand {
    Docker(DockerArgs),
    Mediator(MediatorArgs),
    All(SetupAllArgs),
    Sut(SutArgs),
    Jepsen(JepsenArgs),
}

pub fn run(args: SetupArgs, logger: &Logger, sh: &Shell) -> Result<()> {
    match args.command {
        SetupSubcommand::Docker(args) => docker::run(args, logger, sh)?,
        SetupSubcommand::Mediator(args) => mediator::run(args, logger, sh)?,
        SetupSubcommand::All(args) => all::run(args, logger, sh)?,
        SetupSubcommand::Sut(args) => sut::run(args, logger, sh)?,
        SetupSubcommand::Jepsen(args) => jepsen::run(args, logger, sh)?,
    }
    Ok(())
}
