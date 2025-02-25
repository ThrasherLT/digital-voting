//! Code for collecting traces to a log file.

use std::path::Path;

use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard as TracingWorkerGuard;

/// Start a logger that writes traces to a file without blocking.
///
/// # Errors
///
/// If the path to the log file is invalid.
/// If initializing the `tracing_subscriber` fails.
pub fn start_logger(log_path: &Path) -> std::io::Result<TracingWorkerGuard> {
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let log_file = std::fs::File::create(log_path)?;
    // Set up a rolling file appender
    // Do not let _tracing_worker_guard go out of scope, or the logging thread will be terminated.
    let (non_blocking_tracing_writer, tracing_worker_guard) =
        tracing_appender::non_blocking(log_file);

    tracing_subscriber::fmt()
        // TODO For now allowing all log levels.
        .with_max_level(LevelFilter::TRACE)
        .with_writer(non_blocking_tracing_writer)
        .with_line_number(true)
        .with_ansi(false)
        .with_level(true)
        .try_init()
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to initialize logger {e}"),
            )
        })?;

    Ok(tracing_worker_guard)
}
