//! Module for user interaction with the node via CLI.
//! This API is in an early stage and will probably change as the implementation grows.

// TODO daemonize at least on Unix systems.

use clap::Parser;
use crypto::signature::blind_sign;

/// Command line arguments for the node.
/// All the stuff required to start the node.
#[derive(Parser, Clone, Debug)]
pub struct Args {
    /// The address the node will listen on.
    #[clap(short = 'a', long = "address", default_value = "127.0.0.1:8080")]
    pub socket_addr: std::net::SocketAddr,
    /// The public key of the election authority used to verify that the voters are eligible.
    #[clap(short = 'p', long = "authority-public-key")]
    pub authority_pk: blind_sign::PublicKey,
    /// The command to execute. See `Cmd` for more details.
    #[clap(subcommand)]
    pub cmd: Cmd,
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
