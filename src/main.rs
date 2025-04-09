mod commands;
mod model;
use clap::Parser;
use commands::{Abstraktor, AbstraktorSubcommand};

fn main() {
    let args = Abstraktor::parse();
    match args.command {
        AbstraktorSubcommand::Instrument(args) => commands::instrument::run(args),
    }
}
