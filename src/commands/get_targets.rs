use clap::Parser;
use std::{fs, path};
use std::path::Path;

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
    let path = path::absolute(Path::new(path)).unwrap();

    if path.is_file() {
        if let Ok(content) = fs::read_to_string(&path) {
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
        visit_dirs(&path, &mut files);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::LogLevel;

    fn normalize_and_sort(mut value: serde_json::Value, base_dir: &Path) -> serde_json::Value {
        if let Some(array) = value.as_array_mut() {
            for item in array.iter_mut() {
                if let Some(obj) = item.as_object_mut() {
                    if let Some(path_val) = obj.get_mut("path") {
                        if let Some(p) = path_val.as_str() {
                            let path = Path::new(p);
                            let rel = if path.is_absolute() {
                                pathdiff::diff_paths(path, base_dir).unwrap_or_else(|| path.to_path_buf())
                            } else {
                                path.to_path_buf()
                            };
                            *path_val = serde_json::Value::String(rel.to_string_lossy().into_owned());
                        }
                    }
                }
            }
    
            array.sort_by(|a, b| {
                let a_path = a.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let b_path = b.get("path").and_then(|v| v.as_str()).unwrap_or("");
                a_path.cmp(b_path)
            });
        }
        value
    }

    #[test]
    fn test_get_targets_command() {
        let test_dir = Path::new("./tests/instrument_test");
        let output_file = test_dir.join("targets.json");
        let expected_file = test_dir.join("expected_targets.json");

        if output_file.exists() {
            fs::remove_file(&output_file).unwrap();
        }

        let logger = Logger::new(LogLevel::Quiet);

        let args = GetTargetsArgs {
            path: test_dir.to_string_lossy().into_owned(),
            output: output_file.to_string_lossy().into_owned(),
        };
        run(args, &logger);

        let output = fs::read_to_string(&output_file).unwrap();
        let expected = fs::read_to_string(&expected_file).unwrap();

        let mut output_json: serde_json::Value = serde_json::from_str(&output).unwrap();
        let mut expected_json: serde_json::Value = serde_json::from_str(&expected).unwrap();
        
        let base_dir = test_dir.canonicalize().unwrap();
        output_json = normalize_and_sort(output_json, &base_dir);
        expected_json = normalize_and_sort(expected_json, &base_dir);
        
        assert_eq!(output_json, expected_json);

        fs::remove_file(&output_file).unwrap();
    }
}
