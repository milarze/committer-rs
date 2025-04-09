use clap::Command;
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
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::sync::Mutex;
    use std::time::Instant;
    use std::collections::HashMap;
    use chrono::Local;
    use tracing::{span, Subscriber};
    use tracing_subscriber::{Layer, registry::LookupSpan};
    use once_cell::sync::Lazy;
    
    // Global storage for span entry times (using once_cell for safe static initialization)
    static SPAN_TIMESTAMPS: Lazy<Mutex<HashMap<String, Instant>>> = 
        Lazy::new(|| Mutex::new(HashMap::new()));
    
    // Create a simpler layer that just logs to CSV
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
            
            // Create/check the file
            if !path.exists() {
                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(path)
                    .expect("Failed to create log file");
                
                writeln!(file, "timestamp,function,duration_us")
                    .expect("Failed to write CSV headers");
                
                eprintln!("Created performance log file: {}", log_file_path);
            }
            
            SpanTimingLayer { log_file_path }
        }
    }
    
    impl<S> Layer<S> for SpanTimingLayer 
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
            // When a new span is created, store its start time in our HashMap
            let span_name = attrs.metadata().name().to_string();
            let mut timestamps = SPAN_TIMESTAMPS.lock().unwrap();
            let key = format!("{}-{:?}", span_name, id);
            timestamps.insert(key, Instant::now());
            
            eprintln!("Span started: {} ({:?})", span_name, id);
        }
        
        fn on_close(&self, id: span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
            // Get the span name
            let span = match ctx.span(&id) {
                Some(span) => span,
                None => return,
            };
            
            let span_name = span.name().to_string();
            eprintln!("Span closed: {} ({:?})", span_name, id);
            
            // Get the start time from our HashMap
            let key = format!("{}-{:?}", span_name, id);
            let duration = {
                let timestamps = SPAN_TIMESTAMPS.lock().unwrap();
                if let Some(start_time) = timestamps.get(&key) {
                    let elapsed = start_time.elapsed();
                    eprintln!("Duration for {}: {:?}", span_name, elapsed);
                    elapsed
                } else {
                    eprintln!("No start time found for span: {}", span_name);
                    return;
                }
            };
            
            // Format and write to CSV
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
            let csv_line = format!("{},{},{}\n", timestamp, span_name, duration.as_micros());
            
            // Write to file
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.log_file_path)
            {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(csv_line.as_bytes()) {
                        eprintln!("Failed to write to log file: {}", e);
                    } else {
                        eprintln!("Wrote to CSV: {}", csv_line.trim());
                    }
                },
                Err(e) => eprintln!("Failed to open log file: {}", e),
            }
            
            // Clean up the entry
            let mut timestamps = SPAN_TIMESTAMPS.lock().unwrap();
            timestamps.remove(&key);
        }
    }
    
    // Create the log file path
    let home_dir = home::home_dir().expect("Unable to determine home directory");
    let log_file = home_dir.join(".committer-rs").join("performance.csv");
    let log_file_path = log_file.to_string_lossy().to_string();
    
    // Create our custom timing layer
    let timing_layer = SpanTimingLayer::new(log_file_path);
    
    // Set up the registry with our timing layer and standard console output
    let registry = tracing_subscriber::registry()
        .with(timing_layer)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(tracing_subscriber::EnvFilter::new("info"));
    
    // Initialize the subscriber
    eprintln!("Initializing tracing subscriber");
    registry.init();
    eprintln!("Tracing subscriber initialized - span timings will be written to CSV");
}
