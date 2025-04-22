mod commands;
mod model;
use clap::Parser;
use commands::{Abstraktor, AbstraktorSubcommand};

fn main() {
    let args = Abstraktor::parse();
    match args.command {
        AbstraktorSubcommand::GetTargets(args) => commands::get_targets::run(args),
    }
}
