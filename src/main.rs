use clap::{Arg, Command};

fn main() {
    let matches = Command::new("abstraktor")
        .version("1.0")
        .author("Matias Gonzalez <maigonzalez@fi.uba.ar>")
        .about("Abstraktor")
        .arg(Arg::new("path")
            .long("path")
            .required(true)
            .help("Some path"))
        .get_matches();

    let path = matches.get_one::<String>("path").unwrap();
    println!("{}", path);
}
