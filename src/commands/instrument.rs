use clap::Parser;

use crate::model::instrumentor::Instrumentor;

#[derive(Parser, Debug)]
pub struct InstrumentArgs {
    #[arg(short, long)]
    path: String,
}

pub fn run(args: InstrumentArgs) {
    let _instrumentor = Instrumentor::new();

    println!("{:?}", args);
}
