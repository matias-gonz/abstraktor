use clap::{Parser, Subcommand, command};

pub mod get_targets;
pub use get_targets::GetTargetsArgs;

pub mod llvm;
pub use llvm::LlvmArgs;

pub mod instrument;
pub use instrument::InstrumentArgs;

pub mod setup;
pub use setup::SetupArgs;

pub mod run_mallory;
pub use run_mallory::RunMalloryArgs;

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
	GetTargets(GetTargetsArgs),
	Llvm(LlvmArgs),
	Instrument(InstrumentArgs),
	Setup(SetupArgs),
	RunMallory(RunMalloryArgs),
}
