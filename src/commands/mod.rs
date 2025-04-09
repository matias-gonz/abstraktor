use clap::{command, Parser, Subcommand};

pub mod instrument;
pub use instrument::{InstrumentArgs, run};

#[derive(Parser, Debug)]
#[command(
    name = "abstraktor",
    version = "1.0",
    author = "Matias Gonzalez <maigonzalez@fi.uba.ar>",
    about = "Abstraktor"
)]
pub struct Abstraktor {
    #[command(subcommand)]
    pub command: AbstraktorSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum AbstraktorSubcommand {
    Instrument(InstrumentArgs),
}
