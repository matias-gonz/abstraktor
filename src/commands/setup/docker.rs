use std::process::{Command, Stdio};
use anyhow::{Context, Result};
use clap::Parser;
use crate::logger::Logger;

#[derive(Parser, Debug)]
pub struct DockerArgs {
    // No additional arguments needed for this command
}

pub fn run(_args: DockerArgs, logger: &Logger) -> Result<()> {
    logger.log("Setting up Docker environment...");
    
    let status = Command::new("bash")
        .arg("mallory/docker/bin/up")
        .current_dir(".")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute mallory/docker/bin/up script")?;

    if status.success() {
        logger.success("Docker environment setup completed successfully!");
    } else {
        anyhow::bail!("Docker setup failed with exit code: {}", status);
    }

    Ok(())
} 