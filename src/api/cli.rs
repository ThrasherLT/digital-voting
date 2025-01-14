//! Module for user interaction with the node via CLI.
//! This API is in an early stage and will probably change as the implementation grows.

// TODO daemonize at least on Unix systems.

use std::path::PathBuf;

use clap::Parser;

// TODO make sure paths exist and won't cause an error

/// Command line arguments for the node.
/// All the stuff required to start the node.
#[derive(Parser, Clone, Debug)]
pub struct Args {
    /// The address the node will listen on.
    #[clap(
        long = "address",
        default_value = "0.0.0.0:8080",
        help = "The socket address on which the http server of the node will be hosted"
    )]
    pub socket_addr: std::net::SocketAddr,
    /// Path to where the log, config and etc files are stored.
    #[clap(
        long = "data-path",
        default_value = "./data",
        help = "Path to where log, config and similar files will be stored"
    )]
    pub data_path: PathBuf,
    /// Don't start a CLI interface for this node.
    #[clap(
        long = "no-cli",
        default_value_t = false,
        help = "Only run HTTP server and do not start a CLI interface"
    )]
    pub no_cli: bool,
    /// The command to execute. See `Cmd` for more details.
    #[clap(subcommand)]
    pub cmd: Option<Cmd>,
}

/// The command that the node should execute on startup.
#[derive(Parser, Clone, Debug)]
pub enum Cmd {
    /// Start a new blockchain and create a genesis block.
    #[clap(about = "Start new blokcchain")]
    Genesis {},
    /// Connect to an existing node of an existing blockchain.
    #[clap(about = "Connect to existing node")]
    Connect {
        /// The address of the existing node.
        peer_socket_addr: std::net::SocketAddr,
    },
}
