use std::process::{Command, Stdio};
use anyhow::{Context, Result};
use clap::Parser;
use crate::logger::Logger;

#[derive(Parser, Debug)]
pub struct DockerArgs {
    // No additional arguments needed for this command
}

pub fn run(_args: DockerArgs, logger: &Logger) -> Result<()> {
    logger.log("Building Docker images...");
    
    let status = Command::new("bash")
        .arg("bin/up")
        .arg("--build-only")
        .current_dir("mallory/docker")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute mallory/docker/bin/up script")?;

    if status.success() {
        logger.success("Docker images built successfully!");
    } else {
        anyhow::bail!("Docker build failed with exit code: {}", status);
    }

    Ok(())
} 