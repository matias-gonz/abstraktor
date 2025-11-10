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

	let console_path = path::absolute(Path::new("mallory/docker/bin/console"))
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

	if !console_path.exists() {
		logger.error(format!(
			"Mallory up script not found at {}",
			console_path.to_string_lossy()
		));
		return Err(anyhow::anyhow!(
			"Mallory up script not found at {}",
			console_path.to_string_lossy()
		));
	}

	logger.log("Launching Docker containers and services");
	logger.debug(format!("Executing: bash {}", up_path.to_string_lossy()));
	
	sh.cmd("sudo")
		.arg(up_path.to_string_lossy().as_ref())
		.arg("--no-build")
		.run()
		.context("Failed to run mallory up")?;

	logger.success("Mallory environment is running");
	
	let result = sh.cmd("sudo")
		.arg(console_path.to_string_lossy().as_ref())
		.arg("cd /jepsen/mediator && ./target/x86_64-unknown-linux-musl/release/mediator qlearning event_history 0.7 & sleep 5 && cd /jepsen/tests/mallory/dqlite && lein run test --workload append --nemesis all --time-limit 65 --test-count 1 && cp /jepsen/tests/mallory/dqlite/store/latest/mediator.log /host")
		.run();

	sh.cmd("bash")
		.arg("-c")
		.arg("docker ps -q --filter 'name=jepsen-' | xargs -r docker stop")
		.run()
		.context("Failed to stop Jepsen containers")?;

	sh.cmd("bash")
		.arg("-c")
		.arg("docker ps -a -q --filter 'name=jepsen-' | xargs -r docker rm -f")
		.run()
		.context("Failed to stop Jepsen containers")?;
	 
	Ok(())
	
}

