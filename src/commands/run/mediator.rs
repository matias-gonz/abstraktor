use std::path::{self, Path};

use anyhow::{Context, Result};
use clap::Parser;
use xshell::Shell;

use crate::logger::Logger;

const MEDIATOR_BIN_REL: &str = "mallory/mediator/target/x86_64-unknown-linux-musl/release/mediator";
const ALGORITHM: &str = "qlearning";
const TABLE: &str = "event_history";
const REWARD: &str = "0.7";

#[derive(Parser, Debug)]
pub struct RunMediatorArgs {}

pub fn run(_args: RunMediatorArgs, logger: &Logger, sh: &Shell) -> Result<()> {
	logger.log("Starting Mediator");
	
	let bin_path = path::absolute(Path::new(MEDIATOR_BIN_REL))
		.context("Failed to absolutize mediator binary path")?;
	logger.debug(format!("Mediator binary path: {}", bin_path.display()));
	
	if !bin_path.exists() {
		logger.error(format!(
			"Mediator binary not found at {}",
			bin_path.to_string_lossy()
		));
		logger.log("Hint: Run 'abstraktor setup mediator' to build the mediator first");
		return Err(anyhow::anyhow!(
			"Mediator binary not found at {}",
			bin_path.to_string_lossy()
		));
	}

	logger.log("Launching mediator with configuration:");
	logger.log(format!("  Algorithm: {}", ALGORITHM));
	logger.log(format!("  Table: {}", TABLE));
	logger.log(format!("  Reward: {}", REWARD));
	logger.debug(format!("Executing: sudo {} {} {} {}", 
		bin_path.to_string_lossy(), ALGORITHM, TABLE, REWARD));
	
	sh.cmd("sudo")
		.arg(bin_path.to_string_lossy().as_ref())
		.arg(ALGORITHM)
		.arg(TABLE)
		.arg(REWARD)
		.run()
		.context("Failed to run mediator")?;

	logger.success("Mediator started successfully");
	Ok(())
}

