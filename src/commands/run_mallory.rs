use std::path::{self, Path};
use std::process::Command;

use anyhow::{Context, Result};
use clap::Parser;

use crate::logger::Logger;

#[derive(Parser, Debug)]
pub struct RunMalloryArgs {}

pub fn run(_args: RunMalloryArgs, logger: &Logger) -> Result<()> {
	let up_path = path::absolute(Path::new("mallory/docker/bin/up"))
		.context("Failed to absolutize mallory up path")?;
	if !up_path.exists() {
		logger.error(format!(
			"mallory up not found at {}",
			up_path.to_string_lossy()
		));
		return Err(anyhow::anyhow!(
			"mallory up not found at {}",
			up_path.to_string_lossy()
		));
	}

	logger.log(format!("Running {}", up_path.to_string_lossy()));
	let status = Command::new("sh")
		.arg(up_path)
		.status()
		.context("Failed to run mallory up")?;

	if !status.success() {
		anyhow::bail!("mallory up exited with status: {}", status);
	}

	logger.success("Mallory is up");
	Ok(())
} 