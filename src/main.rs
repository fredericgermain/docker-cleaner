use clap::{Arg, Command};
use std::path::PathBuf;

mod analysis;
mod container;
mod image;
mod overlay2;
mod ui;
mod node;

fn main() -> anyhow::Result<()> {
    let matches = Command::new("docker-cleaner")
        .arg(Arg::new("delete")
            .long("delete")
            .help("Propose to delete files/directories one by one"))
        .arg(Arg::new("dry-run")
            .long("dry-run")
            .help("Display what would happen without actually deleting"))
        .arg(Arg::new("base")
            .long("base")
            .value_name("PATH")
            .default_value("/var/lib/docker")
            .help("Base directory for Docker data"))
        .get_matches();

    let base_path = PathBuf::from(matches.get_one::<String>("base").unwrap());
   // let delete_mode = matches.contains_id("delete");
   // let dry_run = matches.contains_id("dry-run");

    let graph = analysis::build_graph(&base_path)?;
    
    ui::run_ui(graph, base_path)?;

    Ok(())
}