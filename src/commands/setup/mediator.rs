use anyhow::{Context, Result};
use clap::Parser;
use crate::logger::Logger;
use xshell::Shell;

#[derive(Parser, Debug)]
pub struct MediatorArgs {
    #[arg(long, default_value = "false")]
    release: bool,
    
    #[arg(long, default_value = "false")]
    selfcheck: bool,
    
    #[arg(long, default_value = "false")]
    logsaving: bool,
}

pub fn run(args: MediatorArgs, logger: &Logger, sh: &Shell) -> Result<()> {
    logger.log("Setting up Mediator...");
    
    let mediator_dir = "mallory/mediator";
    if !std::path::Path::new(mediator_dir).exists() {
        anyhow::bail!("Mediator directory not found at: {}", mediator_dir);
    }

    let _dir = sh.push_dir(mediator_dir);

    let mut feature_flags: Vec<&str> = Vec::new();
    if args.selfcheck {
        feature_flags.push("selfcheck");
        logger.log("Adding selfcheck feature...");
    }
    if args.logsaving {
        feature_flags.push("logsaving");
        logger.log("Adding logsaving feature...");
    }

    let mut cmd = sh.cmd("cargo");
    cmd = cmd.arg("build");
    if args.release {
        cmd = cmd.arg("--release");
        logger.log("Building in release mode...");
    }
    if !feature_flags.is_empty() {
        cmd = cmd.arg("--features").arg(feature_flags.join(","));
    }
    cmd = cmd.env("RUSTFLAGS", "-C target-cpu=native");

    logger.log("Building mediator with cargo...");
    cmd.run().context("Failed to execute cargo build for mediator")?;

    logger.success("Mediator build completed successfully!");
    Ok(())
} 