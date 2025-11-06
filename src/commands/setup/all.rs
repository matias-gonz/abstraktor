use clap::Parser;
use anyhow::Result;
use crate::logger::Logger;
use super::{docker::DockerArgs, mediator::MediatorArgs};
use xshell::Shell;

#[derive(Parser, Debug)]
pub struct SetupAllArgs {
    #[command(flatten)]
    pub docker: DockerArgs,
    
    #[command(flatten)]
    pub mediator: MediatorArgs,
}

pub fn run(args: SetupAllArgs, logger: &Logger, sh: &Shell) -> Result<()> {
    logger.log("Running complete setup (Docker + Mediator)");
    logger.debug("This will build Docker images and compile the Mediator");
    
    logger.log("Step 1/2: Setting up Docker");
    super::docker::run(args.docker, logger, sh)?;
    
    logger.log("Step 2/2: Setting up Mediator");
    super::mediator::run(args.mediator, logger, sh)?;
    
    logger.success("Complete setup finished successfully!");
    Ok(())
} 