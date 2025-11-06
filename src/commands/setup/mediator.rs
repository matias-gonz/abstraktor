use crate::logger::Logger;
use anyhow::{Context, Result};
use clap::Parser;
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
    logger.log("Setting up Mediator");
    logger.debug(format!("Release mode: {}", args.release));
    logger.debug(format!("Selfcheck feature: {}", args.selfcheck));
    logger.debug(format!("Logsaving feature: {}", args.logsaving));

    let mediator_dir = "mallory/mediator";
    if !std::path::Path::new(mediator_dir).exists() {
        logger.error(format!("Mediator directory not found at: {}", mediator_dir));
        anyhow::bail!("Mediator directory not found at: {}", mediator_dir);
    }

    logger.debug(format!("Changing directory to {}", mediator_dir));
    let _dir = sh.push_dir(mediator_dir);

    let mut feature_flags: Vec<&str> = Vec::new();
    if args.selfcheck {
        feature_flags.push("selfcheck");
        logger.log("Enabling selfcheck feature");
    }
    if args.logsaving {
        feature_flags.push("logsaving");
        logger.log("Enabling logsaving feature");
    }

    let mut cmd = sh.cmd("cargo");
    cmd = cmd.arg("build");

    let build_mode = if args.release {
        cmd = cmd.arg("--release");
        "release"
    } else {
        "debug"
    };
    logger.log(format!("Building in {} mode", build_mode));

    if !feature_flags.is_empty() {
        let features = feature_flags.join(",");
        logger.debug(format!("Features: {}", features));
        cmd = cmd.arg("--features").arg(features);
    }

    cmd = cmd.env("RUSTFLAGS", "-C target-cpu=native");
    logger.debug("RUSTFLAGS set to: -C target-cpu=native");

    logger.log("Compiling mediator (this may take a few minutes)");
    cmd.run()
        .context("Failed to execute cargo build for mediator")?;

    logger.success("Mediator build completed successfully!");
    Ok(())
}
