use std::path::Path;
use std::fs;
use crate::logger::Logger;
use anyhow::{Context, Result};
use clap::Parser;
use xshell::Shell;

const SUT_DIR_NAME: &str = "SUT";
const DOCKER_BUILD_DIR: &str = "mallory/docker/node";

#[derive(Parser, Debug)]
pub struct SutArgs {
    #[arg(short, long)]
    pub path: String,
    
    #[arg(short, long)]
    pub destination: Option<String>,
    
    #[arg(long, default_value = "false")]
    pub rebuild: bool,
}

pub fn run(args: SutArgs, logger: &Logger, sh: &Shell) -> Result<()> {
    logger.log(format!("Copying directory {} to Docker SUT directory", args.path));
    
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
    
    copy_to_sut_directory(source_path, &sut_subdir_name, logger)?;
    
    if args.rebuild {
        rebuild_docker_image(logger, sh)?;
    } else {
        logger.log("Files copied to SUT directory. Use --rebuild flag to rebuild the Docker image with the new files.");
    }
    
    logger.success(format!("Directory copied to SUT/{}", sut_subdir_name));
    Ok(())
}

fn copy_to_sut_directory(source_path: &Path, sut_subdir_name: &str, logger: &Logger) -> Result<()> {
    logger.log("Copying directory to Docker SUT build context...");
    
    let build_dir = Path::new(DOCKER_BUILD_DIR);
    if !build_dir.exists() {
        anyhow::bail!("Docker build directory not found at: {}", DOCKER_BUILD_DIR);
    }
    
    let sut_base_dir = build_dir.join(SUT_DIR_NAME);
    let dest_path = sut_base_dir.join(sut_subdir_name);
    
    if !sut_base_dir.exists() {
        logger.log(format!("Creating SUT directory at {}", sut_base_dir.display()));
        fs::create_dir_all(&sut_base_dir)
            .context("Failed to create SUT directory")?;
    }
    
    if dest_path.exists() {
        logger.log(format!("Removing existing SUT/{} directory", sut_subdir_name));
        fs::remove_dir_all(&dest_path)
            .context("Failed to remove existing SUT subdirectory")?;
    }
    
    logger.log(format!("Copying {} to SUT/{}", source_path.display(), sut_subdir_name));
    
    copy_dir_recursive(source_path, &dest_path)?;
    
    logger.log("Verifying copied files...");
    let entries = fs::read_dir(&dest_path)
        .context("Failed to read destination directory")?;
    
    let mut file_count = 0;
    for entry in entries {
        if entry.is_ok() {
            file_count += 1;
        }
    }
    
    logger.log(format!("Successfully copied {} items to SUT/{}", file_count, sut_subdir_name));
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)
        .context("Failed to create destination directory")?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)
                .context("Failed to copy file")?;
        }
    }
    
    Ok(())
}

fn rebuild_docker_image(logger: &Logger, sh: &Shell) -> Result<()> {
    logger.log("Rebuilding Docker image with copied files...");
    
    let build_dir = Path::new(DOCKER_BUILD_DIR);
    let _dir = sh.push_dir(build_dir);
    sh.cmd("docker")
        .arg("build")
        .arg("-t")
        .arg("jepsen_node")
        .arg(".")
        .run()
        .context("Failed to rebuild Docker image")?;
    
    logger.success("Docker image rebuilt successfully!");
    Ok(())
} 