use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use xshell::Shell;
use crate::logger::Logger;

pub const SUT_DIR_NAME: &str = "SUT";

pub fn copy_to_build_sut_dir(build_dir: &str, source_path: &Path, sut_subdir_name: &str, logger: &Logger) -> Result<()> {
    logger.log("Copying directory to Docker SUT build context...");

    let build_dir_path = Path::new(build_dir);
    if !build_dir_path.exists() {
        anyhow::bail!("Docker build directory not found at: {}", build_dir);
    }

    let sut_base_dir = build_dir_path.join(SUT_DIR_NAME);
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

pub fn rebuild_docker_image(build_dir: &str, image_tag: &str, logger: &Logger, sh: &Shell) -> Result<()> {
    logger.log("Rebuilding Docker image with copied files...");

    let build_dir_path = Path::new(build_dir);
    let _dir = sh.push_dir(build_dir_path);
    sh.cmd("docker")
        .arg("build")
        .arg("-t")
        .arg(image_tag)
        .arg(".")
        .run()
        .context("Failed to rebuild Docker image")?;

    logger.success("Docker image rebuilt successfully!");
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


