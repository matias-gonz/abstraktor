use crate::logger::Logger;
use anyhow::Result;
use clap::Args;
use xshell::{Shell, cmd};

#[derive(Args, Debug)]
pub struct CleanArgs {}

pub fn run(_args: CleanArgs, logger: &Logger, sh: &Shell) -> Result<()> {
    logger.log("Starting Docker cleanup...");

    logger.log("Removing all Docker containers...");
    let containers_result = cmd!(sh, "docker ps -aq").read();
    match containers_result {
        Ok(container_ids) => {
            if !container_ids.trim().is_empty() {
                cmd!(sh, "docker rm -f")
                    .args(container_ids.split_whitespace())
                    .run()?;
                logger.success("All containers removed successfully");
            } else {
                logger.log("No containers to remove");
            }
        }
        Err(_) => {
            logger.error("Failed to list containers or no Docker daemon running");
        }
    }

    logger.log("Removing all Docker volumes...");
    let volumes_result = cmd!(sh, "docker volume ls -q").read();
    match volumes_result {
        Ok(volume_names) => {
            if !volume_names.trim().is_empty() {
                cmd!(sh, "docker volume rm")
                    .args(volume_names.split_whitespace())
                    .run()?;
                logger.success("All volumes removed successfully");
            } else {
                logger.log("No volumes to remove");
            }
        }
        Err(_) => {
            logger.error("Failed to list volumes or no Docker daemon running");
        }
    }

    logger.success("Docker cleanup completed");
    Ok(())
}
