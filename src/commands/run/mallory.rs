use std::path::{self, Path};

use anyhow::{Context, Result};
use clap::Parser;
use xshell::Shell;

use crate::logger::Logger;

#[derive(Parser, Debug)]
pub struct RunMalloryArgs {}

pub fn run(_args: RunMalloryArgs, logger: &Logger, sh: &Shell) -> Result<()> {
	logger.log("Starting Mallory test environment");
	
	let up_path = path::absolute(Path::new("mallory/docker/bin/up"))
		.context("Failed to absolutize mallory up path")?;
	logger.debug(format!("Mallory up script path: {}", up_path.display()));
	
	if !up_path.exists() {
		logger.error(format!(
			"Mallory up script not found at {}",
			up_path.to_string_lossy()
		));
		return Err(anyhow::anyhow!(
			"Mallory up script not found at {}",
			up_path.to_string_lossy()
		));
	}

	logger.log("Launching Docker containers and services");
	logger.debug(format!("Executing: sh {}", up_path.to_string_lossy()));
	
	sh.cmd("sh")
		.arg(up_path.to_string_lossy().as_ref())
		.run()
		.context("Failed to run mallory up")?;

	logger.success("Mallory environment is running");
	Ok(())
}

