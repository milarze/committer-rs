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
            // Print path for debugging
            eprintln!("Setting up CSV logging to: {}", log_file.display());
            
            // Ensure parent directory exists
            if let Some(parent) = log_file.parent() {
                eprintln!("Creating directory: {}", parent.display());
                fs::create_dir_all(parent).expect("Failed to create log directory");
            }
            
            // Write headers if file doesn't exist
            if !log_file.exists() {
                eprintln!("Creating new log file with headers");
                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&log_file)
                    .expect("Failed to create log file");
                
                writeln!(file, "timestamp,function,duration_us")
                    .expect("Failed to write CSV headers");
            } else {
                eprintln!("Log file already exists");
            }
            
            CsvLoggerLayer {
                log_file: Arc::new(Mutex::new(log_file)),
            }
        }
        
        fn log_span(&self, function: &str, duration: std::time::Duration) {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
            let duration_us = duration.as_micros();
            let csv_line = format!("{},{},{}\n", timestamp, function, duration_us);
            
            eprintln!("Writing to CSV: {}", csv_line.trim());
            
            if let Ok(path) = self.log_file.lock() {
                eprintln!("Got lock on path: {}", path.display());
                match OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&*path)
                {
                    Ok(mut file) => {
                        match file.write_all(csv_line.as_bytes()) {
                            Ok(_) => eprintln!("Successfully wrote to file"),
                            Err(e) => eprintln!("Error writing to file: {}", e),
                        }
                    },
                    Err(e) => eprintln!("Error opening file: {}", e),
                }
            } else {
                eprintln!("Failed to get lock on log file path");
            }
        }
    }

    impl<S> Layer<S> for CsvLoggerLayer
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        fn on_close(&self, id: span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
            eprintln!("CsvLoggerLayer::on_close called");
            
            // Get the span that's closing
            if let Some(span) = ctx.span(&id) {
                // Get the span name (function name)
                let name = span.name().to_string();
                eprintln!("Span closed: {}", name);
                
                // Get duration from the span's extensions
                if let Some(duration) = span.extensions().get::<std::time::Duration>() {
                    eprintln!("Duration found: {:?}", duration);
                    // Log the span
                    self.log_span(&name, *duration);
                } else {
                    eprintln!("No duration found in span extensions");
                }
            } else {
                eprintln!("Span not found in context");
            }
        }
        
        // Add an on_new_span method to verify our layer is working
        fn on_new_span(
            &self,
            attrs: &tracing::span::Attributes<'_>,
            _id: &span::Id,
            _ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            eprintln!("CsvLoggerLayer::on_new_span called for span: {}", attrs.metadata().name());
        }
    }
    
    // Create the log file path
    let home_dir = home::home_dir().expect("Unable to determine home directory");
    let log_file = home_dir.join(".committer-rs").join("performance.csv");
    
    // Create our custom layer
    let csv_layer = CsvLoggerLayer::new(log_file);
    
    // Print span info for the command that's about to run
    eprintln!("About to run the command with instrumented spans");

    // Register with the tracing system using the registry
    // IMPORTANT: We need to use a timer to make sure spans have durations
    let registry = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .with_timer(tracing_subscriber::fmt::time::uptime())
            .with_span_events(FmtSpan::CLOSE)  // This is critical for capturing span close events
            .with_target(false)
            .with_thread_ids(true)
            .pretty()) // Use pretty printing for better debug
        .with(csv_layer) // Our custom CSV layer
        .with(tracing_subscriber::EnvFilter::new("info"));
    
    // Initialize the subscriber
    eprintln!("Initializing tracing subscriber");
    registry.init();
    eprintln!("Tracing subscriber initialized");
}
