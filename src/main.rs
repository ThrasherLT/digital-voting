use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};
use clap::Parser;

use digital_voting::{
    api::{cli::Args, config},
    state::State,
};
use process_io::{cli::StdioReader, logging::start_logger};
use tokio::task::JoinHandle;
use tracing::trace;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let _tracing_worker_guard = start_logger(&args.data_path.join("node.log"))?;
    let election_config = config::load_from_file(&args.data_path.join("election-config.json"))
        .await
        .map_err(|e| anyhow!("Failed to load election config: {e}"))?;
    let state = State::new(election_config);
    trace!("Config loaded");

    let (stop_server, server_handle) = digital_voting::api::server::run(state, args.socket_addr)?;
    if !args.no_cli {
        let _cli_handle: JoinHandle<anyhow::Result<()>> = tokio::task::spawn_blocking(|| {
            let mut stdio_reader = StdioReader::new(PathBuf::from_str("node-cmd-history.txt")?)?;
            loop {
                let line = match stdio_reader.read_stdio_blocking() {
                    Ok(line) => line,
                    Err(e) => {
                        println!("Quitting: {e:?}");
                        if let Err(()) = stop_server.send(()) {
                            println!("Server already down");
                        }
                        break Ok(());
                    }
                };
                println!("Read line: {line:?}");
            }
        });
    }
    server_handle.await??;

    Ok(())
}
