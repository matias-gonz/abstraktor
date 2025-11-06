use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use xshell::Shell;

use crate::logger::Logger;
use crate::model::{build_event_graph, dot_for_node_graph};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Dot,
    Png,
    Pdf,
}

#[derive(Parser, Debug)]
pub struct ExportGraphsArgs {
    #[arg(short = 'l', long = "log", default_value = "mediator-logs/events.log")]
    pub log_path: String,

    #[arg(short = 'o', long = "out", default_value = "abstractions")]
    pub output_dir: String,

    #[arg(short = 'f', long = "format", value_enum, default_value = "png")]
    pub format: OutputFormat,
}

pub fn run(args: ExportGraphsArgs, logger: &Logger, sh: &Shell) -> Result<()> {
    let log_content = fs::read_to_string(&args.log_path)
        .with_context(|| format!("reading log from {}", &args.log_path))?;

    let graph = build_event_graph(&log_content);

    let out_dir = Path::new(&args.output_dir);
    if !out_dir.exists() {
        fs::create_dir_all(out_dir)
            .with_context(|| format!("creating output directory {}", &args.output_dir))?;
    }

    for (node_id, node_graph) in &graph.nodes {
        let dot = dot_for_node_graph(node_graph);
        match args.format {
            OutputFormat::Dot => {
                let file_path = out_dir.join(format!("node_{}.dot", node_id));
                fs::write(&file_path, dot)
                    .with_context(|| format!("writing {}", file_path.display()))?;
                logger.success(format!("wrote {}", file_path.display()));
            }
            OutputFormat::Png | OutputFormat::Pdf => {
                let tmp_dot = out_dir.join(format!("node_{}.dot", node_id));
                fs::write(&tmp_dot, &dot)
                    .with_context(|| format!("writing {}", tmp_dot.display()))?;
                let ext = match args.format {
                    OutputFormat::Png => "png",
                    OutputFormat::Pdf => "pdf",
                    _ => unreachable!(),
                };
                let out_path = out_dir.join(format!("node_{}.{}", node_id, ext));
                sh.cmd("dot")
                    .arg(format!("-T{}", ext))
                    .arg(&tmp_dot)
                    .arg("-o")
                    .arg(&out_path)
                    .run()
                    .with_context(|| {
                        format!("graphviz 'dot' failed generating {}", out_path.display())
                    })?;
                let _ = fs::remove_file(&tmp_dot);
                logger.success(format!("wrote {}", out_path.display()));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_logger() -> Logger {
        Logger::new(crate::logger::LogLevel::Quiet)
    }

    #[test]
    fn test_export_dot_format_single_node() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");

        let args = ExportGraphsArgs {
            log_path: "tests/export_graphs_test/simple_events.log".to_string(),
            output_dir: output_dir.to_str().unwrap().to_string(),
            format: OutputFormat::Dot,
        };

        let logger = create_test_logger();
        let sh = Shell::new().unwrap();

        let result = run(args, &logger, &sh);
        assert!(result.is_ok());

        let dot_file = output_dir.join("node_1.dot");
        assert!(dot_file.exists(), "Expected node_1.dot to exist");

        let content = fs::read_to_string(&dot_file).unwrap();
        assert!(content.contains("digraph G{"));
        assert!(content.contains("rankdir=LR;"));
    }

    #[test]
    fn test_export_dot_format_multiple_nodes() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");

        let args = ExportGraphsArgs {
            log_path: "tests/export_graphs_test/multi_node_events.log".to_string(),
            output_dir: output_dir.to_str().unwrap().to_string(),
            format: OutputFormat::Dot,
        };

        let logger = create_test_logger();
        let sh = Shell::new().unwrap();

        let result = run(args, &logger, &sh);
        assert!(result.is_ok());

        assert!(output_dir.join("node_1.dot").exists());
        assert!(output_dir.join("node_2.dot").exists());
        assert!(output_dir.join("node_3.dot").exists());

        let node1_content = fs::read_to_string(output_dir.join("node_1.dot")).unwrap();
        assert!(node1_content.contains("idle"));
        assert!(node1_content.contains("active"));

        let node2_content = fs::read_to_string(output_dir.join("node_2.dot")).unwrap();
        assert!(node2_content.contains("stopped"));
        assert!(node2_content.contains("running"));
    }

    #[test]
    fn test_export_png_format() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");

        let args = ExportGraphsArgs {
            log_path: "tests/export_graphs_test/simple_events.log".to_string(),
            output_dir: output_dir.to_str().unwrap().to_string(),
            format: OutputFormat::Png,
        };

        let logger = create_test_logger();
        let sh = Shell::new().unwrap();

        let result = run(args, &logger, &sh);

        if sh.cmd("which").arg("dot").quiet().run().is_ok() {
            assert!(
                result.is_ok(),
                "Export should succeed when graphviz is installed"
            );

            let png_file = output_dir.join("node_1.png");
            assert!(png_file.exists(), "Expected node_1.png to exist");

            let dot_file = output_dir.join("node_1.dot");
            assert!(!dot_file.exists(), "Temporary .dot file should be removed");
        } else {
            assert!(
                result.is_err(),
                "Export should fail when graphviz is not installed"
            );
        }
    }

    #[test]
    fn test_export_pdf_format() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");

        let args = ExportGraphsArgs {
            log_path: "tests/export_graphs_test/simple_events.log".to_string(),
            output_dir: output_dir.to_str().unwrap().to_string(),
            format: OutputFormat::Pdf,
        };

        let logger = create_test_logger();
        let sh = Shell::new().unwrap();

        let result = run(args, &logger, &sh);

        if sh.cmd("which").arg("dot").quiet().run().is_ok() {
            assert!(
                result.is_ok(),
                "Export should succeed when graphviz is installed"
            );

            let pdf_file = output_dir.join("node_1.pdf");
            assert!(pdf_file.exists(), "Expected node_1.pdf to exist");

            let dot_file = output_dir.join("node_1.dot");
            assert!(!dot_file.exists(), "Temporary .dot file should be removed");
        } else {
            assert!(
                result.is_err(),
                "Export should fail when graphviz is not installed"
            );
        }
    }

    #[test]
    fn test_export_creates_output_directory() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("nested").join("output").join("dir");

        let args = ExportGraphsArgs {
            log_path: "tests/export_graphs_test/simple_events.log".to_string(),
            output_dir: output_dir.to_str().unwrap().to_string(),
            format: OutputFormat::Dot,
        };

        let logger = create_test_logger();
        let sh = Shell::new().unwrap();

        let result = run(args, &logger, &sh);
        assert!(result.is_ok());
        assert!(output_dir.exists());
        assert!(output_dir.join("node_1.dot").exists());
    }

    #[test]
    fn test_export_empty_log() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");

        let args = ExportGraphsArgs {
            log_path: "tests/export_graphs_test/empty_events.log".to_string(),
            output_dir: output_dir.to_str().unwrap().to_string(),
            format: OutputFormat::Dot,
        };

        let logger = create_test_logger();
        let sh = Shell::new().unwrap();

        let result = run(args, &logger, &sh);
        assert!(result.is_ok());

        assert!(output_dir.exists());

        let entries: Vec<_> = fs::read_dir(&output_dir).unwrap().collect();
        assert_eq!(entries.len(), 0, "No files should be created for empty log");
    }

    #[test]
    fn test_export_missing_log_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");

        let args = ExportGraphsArgs {
            log_path: "tests/export_graphs_test/nonexistent.log".to_string(),
            output_dir: output_dir.to_str().unwrap().to_string(),
            format: OutputFormat::Dot,
        };

        let logger = create_test_logger();
        let sh = Shell::new().unwrap();

        let result = run(args, &logger, &sh);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("reading log from"));
    }

    #[test]
    fn test_export_complex_graph() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");

        let args = ExportGraphsArgs {
            log_path: "tests/export_graphs_test/complex_events.log".to_string(),
            output_dir: output_dir.to_str().unwrap().to_string(),
            format: OutputFormat::Dot,
        };

        let logger = create_test_logger();
        let sh = Shell::new().unwrap();

        let result = run(args, &logger, &sh);
        assert!(result.is_ok());

        assert!(output_dir.join("node_1.dot").exists());
        assert!(output_dir.join("node_2.dot").exists());

        let node1_content = fs::read_to_string(output_dir.join("node_1.dot")).unwrap();
        assert!(node1_content.contains("startup"));
        assert!(node1_content.contains("config"));
        assert!(node1_content.contains("ready"));
        assert!(node1_content.contains("network"));
        assert!(node1_content.contains("authenticated"));
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(format!("{:?}", OutputFormat::Dot), "Dot");
        assert_eq!(format!("{:?}", OutputFormat::Png), "Png");
        assert_eq!(format!("{:?}", OutputFormat::Pdf), "Pdf");
    }

    #[test]
    fn test_dot_file_content_structure() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");

        let args = ExportGraphsArgs {
            log_path: "tests/export_graphs_test/simple_events.log".to_string(),
            output_dir: output_dir.to_str().unwrap().to_string(),
            format: OutputFormat::Dot,
        };

        let logger = create_test_logger();
        let sh = Shell::new().unwrap();

        run(args, &logger, &sh).unwrap();

        let content = fs::read_to_string(output_dir.join("node_1.dot")).unwrap();

        assert!(content.starts_with("digraph G{"));
        assert!(content.ends_with("}\n"));
        assert!(content.contains("node [shape=circle"));
        assert!(content.contains("edge [fontsize=9]"));
    }
}
