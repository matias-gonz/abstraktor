use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use crate::logger::Logger;
use xshell::Shell;

#[derive(Parser, Debug)]
pub struct DockerArgs {
	#[arg(long, default_value = "5")]
	pub node_count: usize,
}

pub fn write_config_edn(n: usize) -> Result<()> {
    let path = "mallory/tests/mallory/dqlite/config.edn";

    // Generate the nodes ["n1" "n2" ... "nN"]
    let nodes: Vec<String> = (1..=n)
        .map(|i| format!("\"n{}\"", i))
        .collect();
    
    let content = format!("{{:nodes [{}]}}", nodes.join(" "));

    fs::write(path, content)
        .context(format!("Can't write to {}. That's the folder exists?", path))?;

    Ok(())
}

pub fn run(_args: DockerArgs, logger: &Logger, sh: &Shell) -> Result<()> {
	logger.log("Building Docker images for Mallory");
	write_config_edn(_args.node_count)?;
	logger.debug("Changing directory to mallory/docker");
	
	let _dir = sh.push_dir("mallory/docker");
	logger.debug("Executing: sudo bash bin/up --build-only");
	
	logger.log("Running docker build script (this may take several minutes)");
	sh.cmd("sudo")
		.arg("bash")
		.arg("bin/up")
		.arg("--build-only")
		.arg("--node-count")
		.arg(_args.node_count.to_string())
		.run()
		.context("Failed to execute mallory/docker/bin/up script")?;
	
	logger.success("Docker images built successfully!");
	Ok(())
} 