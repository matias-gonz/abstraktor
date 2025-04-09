use clap::{Arg, Command};
use std::path::Path;
use std::fs;

mod intrumentor;
use intrumentor::Instrumentor;

fn main() {
    let matches = Command::new("abstraktor")
        .version("1.0")
        .author("Matias Gonzalez <maigonzalez@fi.uba.ar>")
        .about("Abstraktor")
        .arg(Arg::new("path")
            .long("path")
            .required(true)
            .help("Path to the file or directory to process"))
        .get_matches();

    let path = matches.get_one::<String>("path").unwrap();
    
    let paths = if Path::new(path).is_dir() {
        // If it's a directory, collect all files
        let mut file_paths = Vec::new();
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    file_paths.push(path.to_string_lossy().to_string());
                }
            }
        }
        file_paths
    } else {
        // If it's a file, just use that
        vec![path.clone()]
    };

    let _instrumentor = Instrumentor::new();
    println!("{:?}", paths);
}
