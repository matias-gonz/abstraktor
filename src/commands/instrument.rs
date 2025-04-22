use clap::Parser;

use crate::model::instrumentor::Instrumentor;

#[derive(Parser, Debug)]
pub struct InstrumentArgs {
    #[arg(short, long)]
    path: String,
}

pub fn run(args: InstrumentArgs) {
    let content = std::fs::read_to_string(&args.path).unwrap();
    let instrumentor = Instrumentor::new();
    let targets = instrumentor.parse_targets(&content, &args.path);
    println!("{:?}", targets);
}
