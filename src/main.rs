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

    // Ensure the log directory exists
    let home_dir = home::home_dir().expect("Unable to determine home directory");
    let log_dir = PathBuf::from(home_dir).join(".committer-rs");
    fs::create_dir_all(&log_dir).expect("Failed to create log directory");

    // Use a single performance log file
    let log_file = log_dir.join("performance.csv");

    // Check if we need to create the file and write headers
    let file_exists = log_file.exists();

    // Create the file writer
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .expect("Failed to open log file");

    // Write CSV headers if this is a new file
    if !file_exists {
        writeln!(file, "timestamp,function,duration_us").expect("Failed to write CSV headers");
    }

    // Create a custom formatter for CSV output
    struct CsvFormatter;

    impl<S, N> tracing_subscriber::fmt::FormatEvent<S, N> for CsvFormatter
    where
        S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
        N: for<'a> tracing_subscriber::fmt::FormatFields<'a> + 'static,
    {
        fn format_event(
            &self,
            ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
            mut writer: tracing_subscriber::fmt::format::Writer<'_>,
            event: &tracing::Event<'_>,
        ) -> std::fmt::Result {
            use std::fmt::Write;

            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");

            // Function name from span or target as fallback
            let function = if let Some(span) = ctx.lookup_current() {
                span.name().to_string()
            } else {
                event.metadata().target().to_string()
            };

            // Extract duration in microseconds
            let mut duration_us = String::new();

            // Define a visitor to extract duration field
            struct DurationVisitor<'a> {
                duration_us: &'a mut String,
            }

            impl<'a> tracing::field::Visit for DurationVisitor<'a> {
                fn record_debug(
                    &mut self,
                    field: &tracing::field::Field,
                    value: &dyn std::fmt::Debug,
                ) {
                    if field.name() == "elapsed_ms" {
                        // Convert milliseconds to microseconds
                        let val_str = format!("{:?}", value);
                        if let Ok(val) = val_str.parse::<f64>() {
                            let us = (val * 1000.0) as u64;
                            write!(self.duration_us, "{}", us).unwrap();
                        }
                    } else if field.name() == "elapsed_us" {
                        write!(self.duration_us, "{:?}", value).unwrap();
                    }
                }
            }

            let mut visitor = DurationVisitor {
                duration_us: &mut duration_us,
            };

            event.record(&mut visitor);

            // Write as CSV
            write!(writer, "{},{},{}", timestamp, function, duration_us)
        }
    }

    // Initialize subscriber with CSV file output
    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_writer(file)
        .with_ansi(false)
        .event_format(CsvFormatter);

    // Create a registry with multiple layers
    tracing_subscriber::registry()
        .with(file_subscriber)
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
