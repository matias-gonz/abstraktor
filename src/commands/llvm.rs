use clap::Parser;


#[derive(Parser, Debug)]
pub struct LlvmArgs {
    #[arg(short, long)]
    path: String,
    #[arg(short, long)]
    targets_path: String,
    #[arg(short, long)]
    output: String,
}

pub fn run(_args: LlvmArgs) {

}

