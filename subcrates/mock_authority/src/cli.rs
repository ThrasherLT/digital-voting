use clap::Parser;

#[derive(Parser, Clone, Debug)]
pub struct Args {
    #[clap(
        long = "address",
        default_value = "0.0.0.0:8080",
        help = "Specify the ip:port on which to host the mock election authority HTTP server"
    )]
    pub addr: std::net::SocketAddr,
    #[clap(
        long = "new-keys",
        default_value_t = false,
        help = "Generate new blind signer keys instead of loading them from FS"
    )]
    pub new_keys: bool,
    #[clap(
        long = "no-http",
        default_value_t = false,
        help = "Only run CLI and do not start an http server"
    )]
    pub no_http_server: bool,
    #[clap(
        long = "no-cli",
        default_value_t = false,
        help = "Only run HTTP server and do not start a CLI interface"
    )]
    pub no_cli: bool,
    #[clap(
        long = "data-path",
        default_value = "./data",
        help = "Path to where log, config and similar files will be stored"
    )]
    pub data_path: std::path::PathBuf,
}

#[derive(Parser, Clone, Debug)]
pub enum Cmd {
    #[clap(about = "Blind sign a blinded message")]
    BlindSign {
        blinded_msg: crypto::signature::blind_sign::BlindedMessage,
    },
    #[clap(about = "Get blinder public key")]
    GetPubkey,
    #[clap(about = "Shut down the mock authority")]
    Quit,
}
