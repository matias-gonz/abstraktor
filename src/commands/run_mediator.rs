use std::path::{self, Path};
use std::process::Command;

use anyhow::{Context, Result};
use clap::Parser;

use crate::logger::Logger;

const MEDIATOR_BIN_REL: &str = "mallory/mediator/target/x86_64-unknown-linux-musl/release/mediator";
const ALGORITHM: &str = "qlearning";
const TABLE: &str = "event_history";
const REWARD: &str = "0.7";

#[derive(Parser, Debug)]
pub struct RunMediatorArgs {}

pub fn run(_args: RunMediatorArgs, logger: &Logger) -> Result<()> {
	let bin_path = path::absolute(Path::new(MEDIATOR_BIN_REL))
		.context("Failed to absolutize mediator binary path")?;
	if !bin_path.exists() {
		logger.error(format!(
			"mediator binary not found at {}",
			bin_path.to_string_lossy()
		));
		return Err(anyhow::anyhow!(
			"mediator binary not found at {}",
			bin_path.to_string_lossy()
		));
	}

	logger.log(format!(
		"Running mediator: {} {} {} {}",
		bin_path.to_string_lossy(),
		ALGORITHM,
		TABLE,
		REWARD
	));
	let status = Command::new("sudo")
		.arg(&bin_path)
		.arg(ALGORITHM)
		.arg(TABLE)
		.arg(REWARD)
		.status()
		.context("Failed to run mediator")?;

	if !status.success() {
		anyhow::bail!("mediator exited with status: {}", status);
	}

	logger.success("Mediator started");
	Ok(())
} 