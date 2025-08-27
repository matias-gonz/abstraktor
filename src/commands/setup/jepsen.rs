use super::sut_common;
use crate::logger::Logger;
use anyhow::Result;
use clap::Parser;
use std::path::Path;
use xshell::Shell;

const DOCKER_BUILD_DIR: &str = "mallory/docker/control";

#[derive(Parser, Debug)]
pub struct JepsenArgs {
    #[arg(short, long)]
    pub path: String,

    #[arg(short, long)]
    pub destination: Option<String>,

    #[arg(long, default_value = "false")]
    pub rebuild: bool,
}

pub fn run(args: JepsenArgs, logger: &Logger, sh: &Shell) -> Result<()> {
    logger.log(format!(
        "Copying directory {} to Docker Jepsen SUT directory",
        args.path
    ));

    let source_path = Path::new(&args.path);
    if !source_path.exists() {
        logger.error(format!("Source path not found at {}", args.path));
        return Err(anyhow::anyhow!("Source path not found at {}", args.path));
    }

    if !source_path.is_dir() {
        logger.error(format!("Path {} is not a directory", args.path));
        return Err(anyhow::anyhow!("Path {} is not a directory", args.path));
    }

    let sut_subdir_name = if let Some(dest) = args.destination {
        dest
    } else {
        source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("source")
            .to_string()
    };

    sut_common::copy_to_build_sut_dir(DOCKER_BUILD_DIR, source_path, &sut_subdir_name, logger)?;

    if args.rebuild {
        sut_common::rebuild_docker_image(DOCKER_BUILD_DIR, "jepsen_control", logger, sh)?;
    } else {
        logger.log("Files copied to Jepsen SUT directory. Use --rebuild flag to rebuild the Docker image with the new files.");
    }

    logger.success(format!(
        "Directory copied to SUT/{} for Jepsen control image",
        sut_subdir_name
    ));
    Ok(())
}
