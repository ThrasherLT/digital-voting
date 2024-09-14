use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard as TracingWorkerGuard;

/// Start a logger that writes traces to a file without blocking.
pub fn start_logger(log_filename: &str) -> std::io::Result<TracingWorkerGuard> {
    // Set up a rolling file appender
    std::fs::create_dir_all("logs")?;
    let log_file = std::fs::File::create(format!("logs/{log_filename}"))?;
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
