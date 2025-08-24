use std::path::{self, Path};

use anyhow::{Context, Result};
use clap::Parser;
use xshell::Shell;

use crate::logger::Logger;

#[derive(Parser, Debug)]
pub struct RunMalloryArgs {}

pub fn run(_args: RunMalloryArgs, logger: &Logger, sh: &Shell) -> Result<()> {
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
	sh.cmd("bash")
		.arg(up_path.to_string_lossy().as_ref())
		.run()
		.context("Failed to run mallory up")?;

	logger.success("Mallory is up");
	Ok(())
}

