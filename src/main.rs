use clap::Command;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use chrono::Local;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::Mutex;
use std::time::Instant;
use tracing::{span, Subscriber};
use tracing_subscriber::{registry::LookupSpan, Layer};

// Global storage for span entry times (using once_cell for safe static initialization)
static SPAN_TIMESTAMPS: Lazy<Mutex<HashMap<String, Instant>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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
    struct SpanTimingLayer {
        log_file_path: String,
    }

    impl SpanTimingLayer {
        fn new(log_file_path: String) -> Self {
            // Create directory if needed
            let path = std::path::Path::new(&log_file_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("Failed to create log directory");
            }

            if !path.exists() {
                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(path)
                    .expect("Failed to create log file");

                writeln!(file, "timestamp,function,duration_us")
                    .expect("Failed to write CSV headers");
            }

            SpanTimingLayer { log_file_path }
        }
    }

    impl<S> Layer<S> for SpanTimingLayer
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        fn on_new_span(
            &self,
            attrs: &span::Attributes<'_>,
            id: &span::Id,
            _ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            let span_name = attrs.metadata().name().to_string();
            let mut timestamps = SPAN_TIMESTAMPS.lock().unwrap();
            let key = format!("{}-{:?}", span_name, id);
            timestamps.insert(key, Instant::now());
        }

        fn on_close(&self, id: span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
            let span = match ctx.span(&id) {
                Some(span) => span,
                None => return,
            };

            let span_name = span.name().to_string();

            let key = format!("{}-{:?}", span_name, id);
            let duration = {
                let timestamps = SPAN_TIMESTAMPS.lock().unwrap();
                if let Some(start_time) = timestamps.get(&key) {
                    let elapsed = start_time.elapsed();
                    elapsed
                } else {
                    return;
                }
            };

            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
            let csv_line = format!("{},{},{}\n", timestamp, span_name, duration.as_micros());

            // Write to file
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.log_file_path)
            {
                // Write to file, but ignore errors.
                Ok(mut file) => file
                    .write_all(csv_line.as_bytes())
                    .ok()
                    .or(Some(()))
                    .unwrap(),
                Err(e) => eprintln!("Failed to open log file: {}", e),
            }

            let mut timestamps = SPAN_TIMESTAMPS.lock().unwrap();
            timestamps.remove(&key);
        }
    }

    let home_dir = home::home_dir().expect("Unable to determine home directory");
    let log_file = home_dir.join(".committer-rs").join("performance.csv");
    let log_file_path = log_file.to_string_lossy().to_string();

    let timing_layer = SpanTimingLayer::new(log_file_path);

    let registry = tracing_subscriber::registry()
        .with(timing_layer)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(tracing_subscriber::EnvFilter::new("info"));

    registry.init();
}
