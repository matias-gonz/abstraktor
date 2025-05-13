use clap::Parser;

use crate::{logger::Logger, model::instrumentor::Instrumentor};

#[derive(Parser, Debug)]
pub struct GetTargetsArgs {
    #[arg(short, long)]
    pub path: String,
    #[arg(short, long)]
    pub output: String,
}

pub fn run(args: GetTargetsArgs, logger: &Logger) {
    logger.log(format!("Getting targets from {}", args.path));
    let content = std::fs::read_to_string(&args.path).unwrap();
    let instrumentor = Instrumentor::new();
    let targets = instrumentor.get_targets(&content, &args.path);
    let targets_json = serde_json::to_string_pretty(&targets).unwrap();
    std::fs::write(&args.output, targets_json).unwrap();
    logger.success(format!("Targets saved to {}", args.output));
}
