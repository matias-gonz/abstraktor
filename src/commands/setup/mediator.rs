use std::process::{Command, Stdio};
use anyhow::{Context, Result};
use clap::Parser;
use crate::logger::Logger;

#[derive(Parser, Debug)]
pub struct MediatorArgs {
    #[arg(long, default_value = "false")]
    release: bool,
    
    #[arg(long, default_value = "false")]
    selfcheck: bool,
    
    #[arg(long, default_value = "false")]
    logsaving: bool,
}

pub fn run(args: MediatorArgs, logger: &Logger) -> Result<()> {
    logger.log("Setting up Mediator...");
    
    let mediator_dir = "mallory/mediator";
    if !std::path::Path::new(mediator_dir).exists() {
        anyhow::bail!("Mediator directory not found at: {}", mediator_dir);
    }
    
    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd.arg("build");
    
    if args.release {
        cargo_cmd.arg("--release");
        logger.log("Building in release mode...");
    }
    
    let mut features = Vec::new();
    if args.selfcheck {
        features.push("selfcheck");
        logger.log("Adding selfcheck feature...");
    }
    if args.logsaving {
        features.push("logsaving");
        logger.log("Adding logsaving feature...");
    }
    
    if !features.is_empty() {
        cargo_cmd.arg("--features");
        cargo_cmd.arg(features.join(","));
    }
    
    cargo_cmd.env("RUSTFLAGS", "-C target-cpu=native");
    cargo_cmd.current_dir(mediator_dir);
    
    cargo_cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    
    logger.log("Building mediator with cargo...");
    
    let status = cargo_cmd
        .status()
        .context("Failed to execute cargo build for mediator")?;

    if status.success() {
        logger.success("Mediator build completed successfully!");
    } else {
        anyhow::bail!("Mediator build failed with exit code: {}", status);
    }

    Ok(())
} 