use clap::Command;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
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
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use chrono::Local;
    use tracing::{span, Subscriber};
    use tracing_subscriber::Layer;

    // CSV logger for span timings
    struct CsvLoggerLayer {
        log_file: Arc<Mutex<PathBuf>>,
    }

    impl CsvLoggerLayer {
        fn new(log_file: PathBuf) -> Self {
            // Ensure parent directory exists
            if let Some(parent) = log_file.parent() {
                fs::create_dir_all(parent).expect("Failed to create log directory");
            }
            
            // Write headers if file doesn't exist
            if !log_file.exists() {
                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&log_file)
                    .expect("Failed to create log file");
                
                writeln!(file, "timestamp,function,duration_us")
                    .expect("Failed to write CSV headers");
            }
            
            CsvLoggerLayer {
                log_file: Arc::new(Mutex::new(log_file)),
            }
        }
        
        fn log_span(&self, function: &str, duration: std::time::Duration) {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
            let duration_us = duration.as_micros();
            let csv_line = format!("{},{},{}\n", timestamp, function, duration_us);
            
            if let Ok(path) = self.log_file.lock() {
                if let Ok(mut file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&*path)
                {
                    let _ = file.write_all(csv_line.as_bytes());
                }
            }
        }
    }

    impl<S> Layer<S> for CsvLoggerLayer
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        fn on_close(&self, id: span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
            // Get the span that's closing
            if let Some(span) = ctx.span(&id) {
                // Get the span name (function name)
                let name = span.name().to_string();
                
                // Get duration from the span's extensions
                if let Some(duration) = span.extensions().get::<std::time::Duration>() {
                    // Log the span
                    self.log_span(&name, *duration);
                }
            }
        }
    }
    
    // Create the log file path
    let home_dir = home::home_dir().expect("Unable to determine home directory");
    let log_file = home_dir.join(".committer-rs").join("performance.csv");
    
    // Create our custom layer
    let csv_layer = CsvLoggerLayer::new(log_file);
    
    // Register with the tracing system using the registry
    let registry = tracing_subscriber::registry()
        .with(csv_layer)
        .with(tracing_subscriber::EnvFilter::new("info"));

    // Set up the console output
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_thread_ids(true);

    // Initialize the subscriber
    registry.with(fmt_layer).init();
}
