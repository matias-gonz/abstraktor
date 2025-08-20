use anyhow::{Context, Result};
use clap::Parser;
use crate::logger::Logger;
use xshell::Shell;

#[derive(Parser, Debug)]
pub struct DockerArgs {}

pub fn run(_args: DockerArgs, logger: &Logger, sh: &Shell) -> Result<()> {
	logger.log("Building Docker images...");
	let _dir = sh.push_dir("mallory/docker");
	sh.cmd("bash")
		.arg("bin/up")
		.arg("--build-only")
		.run()
		.context("Failed to execute mallory/docker/bin/up script")?;
	logger.success("Docker images built successfully!");
	Ok(())
} 