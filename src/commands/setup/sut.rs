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
    logger.log(format!("Setting up SUT from {}", args.path));
    logger.debug(format!("Source path: {}", args.path));
    logger.debug(format!("Rebuild flag: {}", args.rebuild));
    
    let source_path = Path::new(&args.path);
    if !source_path.exists() {
        logger.error(format!("Source path not found: {}", args.path));
        return Err(anyhow::anyhow!("Source path not found at {}", args.path));
    }
    
    if !source_path.is_dir() {
        logger.error(format!("Path is not a directory: {}", args.path));
        return Err(anyhow::anyhow!("Path {} is not a directory", args.path));
    }
    
    let sut_subdir_name = if let Some(dest) = args.destination {
        logger.debug(format!("Using custom destination name: {}", dest));
        dest
    } else {
        let default_name = source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("source")
            .to_string();
        logger.debug(format!("Using source directory name as destination: {}", default_name));
        default_name
    };
    
    copy_to_sut_directory(source_path, &sut_subdir_name, logger)?;
    
    if args.rebuild {
        logger.log("Rebuild flag set - rebuilding Docker image");
        rebuild_docker_image(logger, sh)?;
    } else {
        logger.log("Skipping Docker rebuild (use --rebuild to rebuild the image)");
    }
    
    logger.success(format!("SUT directory ready at SUT/{}", sut_subdir_name));
    Ok(())
}

fn copy_to_sut_directory(source_path: &Path, sut_subdir_name: &str, logger: &Logger) -> Result<()> {
    logger.log("Copying files to Docker SUT build context");
    logger.debug(format!("Destination: SUT/{}", sut_subdir_name));
    
    let build_dir = Path::new(DOCKER_BUILD_DIR);
    if !build_dir.exists() {
        logger.error(format!("Docker build directory not found: {}", DOCKER_BUILD_DIR));
        anyhow::bail!("Docker build directory not found at: {}", DOCKER_BUILD_DIR);
    }
    logger.debug(format!("Build directory: {}", build_dir.display()));
    
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
    
    logger.log(format!("Copying files from {} to SUT/{}", source_path.display(), sut_subdir_name));
    
    copy_dir_recursive(source_path, &dest_path)?;
    
    logger.debug("Verifying copied files");
    let entries = fs::read_dir(&dest_path)
        .context("Failed to read destination directory")?;
    
    let mut file_count = 0;
    for entry in entries {
        if entry.is_ok() {
            file_count += 1;
        }
    }
    
    logger.success(format!("Copied {} items to SUT/{}", file_count, sut_subdir_name));
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
    logger.log("Rebuilding Docker image with copied files");
    logger.debug("This may take several minutes");
    
    let build_dir = Path::new(DOCKER_BUILD_DIR);
    logger.debug(format!("Build directory: {}", build_dir.display()));
    
    let _dir = sh.push_dir(build_dir);
    logger.debug("Running: docker build -t jepsen_node .");
    
    sh.cmd("docker")
        .arg("build")
        .arg("-t")
        .arg("jepsen_node")
        .arg(".")
        .run()
        .context("Failed to rebuild Docker image")?;
    
    logger.success("Docker image 'jepsen_node' rebuilt successfully!");
    Ok(())
} 