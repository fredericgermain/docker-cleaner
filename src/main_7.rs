use clap::{App, Arg};
use std::path::PathBuf;

mod image;

fn main() {
    let matches = App::new("Docker Dangling File Cleaner")
        .version("1.0")
        .author("Your Name")
        .about("Identifies and optionally deletes dangling Docker files")
        .arg(
            Arg::with_name("base")
                .short("b")
                .long("base")
                .value_name("BASE_DIR")
                .help("Sets the base directory (default: /var/lib/docker)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("delete")
                .short("d")
                .long("delete")
                .help("Enables interactive deletion of dangling files"),
        )
        .get_matches();

    let base_dir = matches
        .value_of("base")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/var/lib/docker"));

    let delete_mode = matches.is_present("delete");

    match image::process_docker_directory(&base_dir, delete_mode) {
        Ok(_) => println!("Analysis complete."),
        Err(e) => eprintln!("Error: {}", e),
    }
}