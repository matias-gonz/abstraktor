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
