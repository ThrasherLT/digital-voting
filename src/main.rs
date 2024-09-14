use anyhow::Result;
use clap::Parser;

use digital_voting::{
    api::server_cli::{Args, StdioReader},
    logging::start_logger,
};

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
                    println!("Quitting: {e:?}, send interrupt again to kill the server");
                    break;
                }
            };
            println!("Read line: {line:?}");
        }
    });

    digital_voting::api::server::run(args.socket_addr).await?;

    Ok(())
}
