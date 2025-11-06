use clap::{command, Parser, Subcommand};

pub mod get_targets;
pub use get_targets::GetTargetsArgs;

pub mod llvm;
pub use llvm::LlvmArgs;

pub mod instrument;
pub use instrument::InstrumentArgs;

pub mod setup;
pub use setup::SetupArgs;

pub mod run;
pub use run::RunArgs;

pub mod export_graphs;
pub use export_graphs::ExportGraphsArgs;

#[derive(Parser, Debug)]
#[command(
    name = "abstraktor",
    version = "1.0",
    author = "Matias Gonzalez <maigonzalez@fi.uba.ar>",
    about = "Abstraktor"
)]
pub struct Abstraktor {
    #[arg(
        short = 'l',
        long = "log-level",
        global = true,
        default_value = "info",
        help = "Set the logging level"
    )]
    pub log_level: crate::logger::LogLevel,

    #[command(subcommand)]
    pub command: AbstraktorSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum AbstraktorSubcommand {
    GetTargets(GetTargetsArgs),
    Llvm(LlvmArgs),
    Instrument(InstrumentArgs),
    Setup(SetupArgs),
    Run(RunArgs),
    ExportGraphs(ExportGraphsArgs),
}
