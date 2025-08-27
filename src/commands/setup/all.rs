use super::{docker::DockerArgs, mediator::MediatorArgs};
use crate::logger::Logger;
use anyhow::Result;
use clap::Parser;
use xshell::Shell;

#[derive(Parser, Debug)]
pub struct SetupAllArgs {
    #[command(flatten)]
    pub docker: DockerArgs,

    #[command(flatten)]
    pub mediator: MediatorArgs,
}

pub fn run(args: SetupAllArgs, logger: &Logger, sh: &Shell) -> Result<()> {
    logger.log("Running complete setup (docker + mediator)...");
    super::docker::run(args.docker, logger, sh)?;
    super::mediator::run(args.mediator, logger, sh)?;
    logger.success("Complete setup finished successfully!");
    Ok(())
}
