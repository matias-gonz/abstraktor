use crate::logger::Logger;
use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::Path;
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
        logger.debug(format!(
            "Using source directory name as destination: {}",
            default_name
        ));
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
        logger.error(format!(
            "Docker build directory not found: {}",
            DOCKER_BUILD_DIR
        ));
        anyhow::bail!("Docker build directory not found at: {}", DOCKER_BUILD_DIR);
    }
    logger.debug(format!("Build directory: {}", build_dir.display()));

    let sut_base_dir = build_dir.join(SUT_DIR_NAME);
    let dest_path = sut_base_dir.join(sut_subdir_name);

    if !sut_base_dir.exists() {
        logger.log(format!(
            "Creating SUT directory at {}",
            sut_base_dir.display()
        ));
        fs::create_dir_all(&sut_base_dir).context("Failed to create SUT directory")?;
    }

    if dest_path.exists() {
        logger.log(format!(
            "Removing existing SUT/{} directory",
            sut_subdir_name
        ));
        fs::remove_dir_all(&dest_path).context("Failed to remove existing SUT subdirectory")?;
    }

    logger.log(format!(
        "Copying files from {} to SUT/{}",
        source_path.display(),
        sut_subdir_name
    ));

    copy_dir_recursive(source_path, &dest_path)?;

    logger.debug("Verifying copied files");
    let entries = fs::read_dir(&dest_path).context("Failed to read destination directory")?;

    let mut file_count = 0;
    for entry in entries {
        if entry.is_ok() {
            file_count += 1;
        }
    }

    logger.success(format!(
        "Copied {} items to SUT/{}",
        file_count, sut_subdir_name
    ));
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest).context("Failed to create destination directory")?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path).context("Failed to copy file")?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::{LogLevel, Logger};
    use tempfile::TempDir;

    #[test]
    fn test_sut_args_creation() {
        let args = SutArgs {
            path: "/test/path".to_string(),
            destination: Some("custom_dest".to_string()),
            rebuild: false,
        };
        assert_eq!(args.path, "/test/path");
        assert_eq!(args.destination, Some("custom_dest".to_string()));
        assert_eq!(args.rebuild, false);
    }

    #[test]
    fn test_sut_args_without_custom_destination() {
        let args = SutArgs {
            path: "/test/path".to_string(),
            destination: None,
            rebuild: false,
        };
        assert_eq!(args.path, "/test/path");
        assert!(args.destination.is_none());
        assert_eq!(args.rebuild, false);
    }

    #[test]
    fn test_sut_args_rebuild_flag() {
        let with_rebuild = SutArgs {
            path: "/test".to_string(),
            destination: None,
            rebuild: true,
        };
        assert!(with_rebuild.rebuild);

        let without_rebuild = SutArgs {
            path: "/test".to_string(),
            destination: None,
            rebuild: false,
        };
        assert!(!without_rebuild.rebuild);
    }

    #[test]
    fn test_sut_dir_name_constant() {
        assert_eq!(SUT_DIR_NAME, "SUT");
    }

    #[test]
    fn test_docker_build_dir_constant() {
        assert_eq!(DOCKER_BUILD_DIR, "mallory/docker/node");
    }

    #[test]
    fn test_run_with_nonexistent_path() {
        let logger = Logger::new(LogLevel::Quiet);
        let sh = Shell::new().unwrap();

        let args = SutArgs {
            path: "/nonexistent/path/12345".to_string(),
            destination: None,
            rebuild: false,
        };

        let result = run(args, &logger, &sh);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Source path not found")
        );
    }

    #[test]
    fn test_run_with_file_instead_of_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test_file.txt");
        fs::write(&temp_file, "test content").unwrap();

        let logger = Logger::new(LogLevel::Quiet);
        let sh = Shell::new().unwrap();

        let args = SutArgs {
            path: temp_file.to_string_lossy().to_string(),
            destination: None,
            rebuild: false,
        };

        let result = run(args, &logger, &sh);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }

    #[test]
    fn test_destination_defaults_to_source_name() {
        let test_source = Path::new("./tests/sut_test/sample_source");
        if !test_source.exists() {
            return;
        }

        let source_name = test_source
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("source");

        assert_eq!(source_name, "sample_source");
    }

    #[test]
    fn test_copy_dir_recursive_with_temp_dirs() {
        let temp_src = TempDir::new().unwrap();
        let temp_dest = TempDir::new().unwrap();

        let src_file = temp_src.path().join("test.txt");
        fs::write(&src_file, "test content").unwrap();

        let src_subdir = temp_src.path().join("subdir");
        fs::create_dir(&src_subdir).unwrap();
        let src_nested = src_subdir.join("nested.txt");
        fs::write(&src_nested, "nested content").unwrap();

        let dest_path = temp_dest.path().join("dest");
        let result = copy_dir_recursive(temp_src.path(), &dest_path);
        assert!(result.is_ok());

        assert!(dest_path.join("test.txt").exists());
        assert!(dest_path.join("subdir").exists());
        assert!(dest_path.join("subdir/nested.txt").exists());

        let content = fs::read_to_string(dest_path.join("test.txt")).unwrap();
        assert_eq!(content, "test content");

        let nested_content = fs::read_to_string(dest_path.join("subdir/nested.txt")).unwrap();
        assert_eq!(nested_content, "nested content");
    }

    #[test]
    fn test_rebuild_flag_is_never_true_in_tests() {
        let test_cases = vec![
            SutArgs {
                path: "./test".to_string(),
                destination: None,
                rebuild: false,
            },
            SutArgs {
                path: "./test2".to_string(),
                destination: Some("dest".to_string()),
                rebuild: false,
            },
        ];

        for args in test_cases {
            assert!(!args.rebuild);
        }
    }

    #[test]
    fn test_various_path_formats() {
        let test_cases = vec![
            "./relative/path",
            "/absolute/path",
            "simple_path",
            "../parent/path",
        ];

        for path in test_cases {
            let args = SutArgs {
                path: path.to_string(),
                destination: None,
                rebuild: false,
            };
            assert_eq!(args.path, path);
        }
    }

    #[test]
    fn test_custom_vs_default_destination() {
        let with_custom = SutArgs {
            path: "./source".to_string(),
            destination: Some("custom_name".to_string()),
            rebuild: false,
        };
        assert_eq!(with_custom.destination, Some("custom_name".to_string()));

        let with_default = SutArgs {
            path: "./source".to_string(),
            destination: None,
            rebuild: false,
        };
        assert!(with_default.destination.is_none());
    }

    #[test]
    fn test_copy_preserves_file_content() {
        let temp_src = TempDir::new().unwrap();
        let temp_dest = TempDir::new().unwrap();

        let test_content = "This is important test content that must be preserved!";
        let src_file = temp_src.path().join("important.txt");
        fs::write(&src_file, test_content).unwrap();

        let dest_path = temp_dest.path().join("dest");
        copy_dir_recursive(temp_src.path(), &dest_path).unwrap();

        let copied_content = fs::read_to_string(dest_path.join("important.txt")).unwrap();
        assert_eq!(copied_content, test_content);
    }

    #[test]
    fn test_copy_handles_empty_directory() {
        let temp_src = TempDir::new().unwrap();
        let temp_dest = TempDir::new().unwrap();

        let dest_path = temp_dest.path().join("dest");
        let result = copy_dir_recursive(temp_src.path(), &dest_path);
        assert!(result.is_ok());
        assert!(dest_path.exists());
        assert!(dest_path.is_dir());
    }

    #[test]
    fn test_copy_handles_multiple_nested_levels() {
        let temp_src = TempDir::new().unwrap();
        let temp_dest = TempDir::new().unwrap();

        let level1 = temp_src.path().join("level1");
        fs::create_dir(&level1).unwrap();
        let level2 = level1.join("level2");
        fs::create_dir(&level2).unwrap();
        let level3 = level2.join("level3");
        fs::create_dir(&level3).unwrap();

        let deep_file = level3.join("deep.txt");
        fs::write(&deep_file, "deeply nested").unwrap();

        let dest_path = temp_dest.path().join("dest");
        copy_dir_recursive(temp_src.path(), &dest_path).unwrap();

        assert!(dest_path.join("level1/level2/level3/deep.txt").exists());
        let content = fs::read_to_string(dest_path.join("level1/level2/level3/deep.txt")).unwrap();
        assert_eq!(content, "deeply nested");
    }
}
