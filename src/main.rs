use anyhow::Result;
use clap::Parser;

use digital_voting::{api::server_cli::Args, logging::start_logger};
use process_io::cli::StdioReader;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("Args: {args:?}");
    let _tracing_worker_guard = start_logger("digital_voting.log")?;

    tokio::task::spawn_blocking(|| {
        let mut stdio_reader = StdioReader::new().unwrap();
        loop {
            let line = match stdio_reader.read_stdio_blocking() {
                Ok(line) => line,
                Err(e) => {
                    // TODO
                    println!("Quitting: {e:?}, send interrupt again to kill the server (WIP)");
                    break;
                }
            };
            println!("Read line: {line:?}");
        }
    });

    digital_voting::api::server::run(args.socket_addr).await?;

    Ok(())
}
