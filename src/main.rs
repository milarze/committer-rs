use clap::Command;

fn main() {
    let matches = Command::new("committer-rs")
        .about("A simple CLI tool for generating commit messages")
        .version("0.0.1")
        .subcommand(Command::new("config").about("Configures stuff"))
        .get_matches();

    match matches.subcommand() {
        Some(("config", _)) => {
            committer_rs::commands::configure();
        }
        _ => {
            committer_rs::commands::generate();
        }
    }
}
