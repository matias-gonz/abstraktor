use anyhow::{Context, Result};
use clap::Parser;
use crate::logger::Logger;
use xshell::Shell;

#[derive(Parser, Debug)]
pub struct DockerArgs {
	#[arg(long, default_value = "5")]
	pub node_count: usize,
}


//FIX
pub fn escribir_config_edn(n: usize) -> Result<()> {
    // Definimos la ruta exacta
    let ruta = "mallory/tests/dqlite/config.edn";

    // Generamos la lista de nodos ["n1" "n2" ... "nN"]
    let nodos: Vec<String> = (1..=n)
        .map(|i| format!("\"n{}\"", i))
        .collect();
    
    let contenido = format!("{{:nodes [{}]}}", nodos.join(" "));

    // Escribimos directamente. Si la carpeta no existe, esto fallará.
    fs::write(ruta, contenido)
        .context(format!("No se pudo escribir en {}. ¿Existe la carpeta?", ruta))?;

    Ok(())
}

pub fn run(_args: DockerArgs, logger: &Logger, sh: &Shell) -> Result<()> {
	logger.log("Building Docker images for Mallory");
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