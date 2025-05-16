use clap::Parser;
use std::path::Path;
use std::fs;

use crate::{logger::Logger, model::instrumentor::Instrumentor};

#[derive(Parser, Debug)]
pub struct GetTargetsArgs {
    #[arg(short, long)]
    pub path: String,
    #[arg(short, long)]
    pub output: String,
}

fn get_files_content(path: &str) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let path = Path::new(path);

    if path.is_file() {
        if let Ok(content) = fs::read_to_string(path) {
            files.push((content, path.to_string_lossy().into_owned()));
        }
    } else if path.is_dir() {
        fn visit_dirs(dir: &Path, files: &mut Vec<(String, String)>) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "c" || ext == "cpp" {
                                if let Ok(content) = fs::read_to_string(&path) {
                                    files.push((content, path.to_string_lossy().into_owned()));
                                }
                            }
                        }
                    } else if path.is_dir() {
                        visit_dirs(&path, files);
                    }
                }
            }
        }
        visit_dirs(path, &mut files);
    }

    files
}

pub fn run(args: GetTargetsArgs, logger: &Logger) {
    logger.log(format!("Getting targets from {}", args.path));
    
    let files = get_files_content(&args.path);
    let instrumentor = Instrumentor::new();
    let targets = instrumentor.get_targets(files);
    let targets_json = serde_json::to_string_pretty(&targets).unwrap();
    std::fs::write(&args.output, targets_json).unwrap();
    logger.success(format!("Targets saved to {}", args.output));
}
