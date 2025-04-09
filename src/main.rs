use clap::Command;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn main() {
    setup_profiling();
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

fn setup_profiling() {
    use chrono::Local;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    // Create a console subscriber with standard formatting
    let console_subscriber = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(false);

    // Create a registry with multiple layers
    tracing_subscriber::registry()
        .with(console_subscriber)
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_timer(tracing_subscriber::fmt::time::uptime())
                .with_thread_ids(true)
                .compact(),
        )
        .init();
}
