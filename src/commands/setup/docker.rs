use anyhow::{Context, Result};
use clap::Parser;
use crate::logger::Logger;
use xshell::Shell;

#[derive(Parser, Debug)]
pub struct DockerArgs {}

pub fn run(_args: DockerArgs, logger: &Logger, sh: &Shell) -> Result<()> {
	logger.log("Building Docker images for Mallory");
	logger.debug("Changing directory to mallory/docker");
	
	let _dir = sh.push_dir("mallory/docker");
	logger.debug("Executing: sudo bash bin/up --build-only");
	
	logger.log("Running docker build script (this may take several minutes)");
	sh.cmd("sudo")
		.arg("bash")
		.arg("bin/up")
		.arg("--build-only")
		.run()
		.context("Failed to execute mallory/docker/bin/up script")?;
	
	logger.success("Docker images built successfully!");
	Ok(())
} 