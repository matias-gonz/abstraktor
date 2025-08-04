use clap::Parser;
use anyhow::Result;
use crate::logger::Logger;
use super::{docker::DockerArgs, mediator::MediatorArgs};

#[derive(Parser, Debug)]
pub struct SetupAllArgs {
    #[command(flatten)]
    pub docker: DockerArgs,
    
    #[command(flatten)]
    pub mediator: MediatorArgs,
}

pub fn run(args: SetupAllArgs, logger: &Logger) -> Result<()> {
    logger.log("Running complete setup (docker + mediator)...");
    super::docker::run(args.docker, logger)?;
    super::mediator::run(args.mediator, logger)?;
    logger.success("Complete setup finished successfully!");
    Ok(())
} 