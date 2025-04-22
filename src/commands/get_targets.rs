use clap::Parser;

use crate::model::instrumentor::Instrumentor;

#[derive(Parser, Debug)]
pub struct GetTargetsArgs {
    #[arg(short, long)]
    path: String,
}

pub fn run(args: GetTargetsArgs) {
    let content = std::fs::read_to_string(&args.path).unwrap();
    let instrumentor = Instrumentor::new();
    let targets = instrumentor.get_targets(&content, &args.path);
    println!("{:?}", targets);
}
