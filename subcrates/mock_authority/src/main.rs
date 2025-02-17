//! This is a mock of the election authorities servers which will be responsible for ensuring the
//! eligibility of the voters by signing their public keys. This is only used for testing purposes.

use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, bail, Result};
use clap::Parser;

use crypto::signature::blind_sign;
use process_io::{cli::StdioReader, logging::start_logger};

mod cli;
mod server;

use cli::{Args, Cmd};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AuthorityConfig {
    pk: blind_sign::PublicKey,
    sk: blind_sign::SecretKey,
}

impl AuthorityConfig {
    fn load_from_fs(path: &Path) -> Result<Self> {
        Ok(serde_json::from_slice(&std::fs::read(path)?)?)
    }

    fn save_to_fs(&self, path: &Path) -> Result<()> {
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?
            .write_all(&serde_json::to_vec(self)?)?;

        Ok(())
    }
}

fn new_blind_signer(path: &Path) -> Result<blind_sign::BlindSigner> {
    let blind_signer = blind_sign::BlindSigner::new()?;
    AuthorityConfig {
        pk: blind_signer.get_public_key()?,
        sk: blind_signer.get_secret_key()?,
    }
    .save_to_fs(path)?;

    Ok(blind_signer)
}

fn setup_blind_signer(
    new_keys: bool,
    authority_config_path: &Path,
) -> Result<blind_sign::BlindSigner> {
    if let Some(parent) = authority_config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if new_keys {
        if let Err(err) = std::fs::remove_file(authority_config_path) {
            // It's not an error if the file actually doesn't exist, since we're deleting it anyway.
            if err.kind() != std::io::ErrorKind::NotFound {
                anyhow::bail!("Failed to delete old blind signer cfg file: {}", err);
            }
        }
    }

    match load_blind_signer_from_fs(authority_config_path) {
        Ok(blind_signer) => Ok(blind_signer),
        Err(_) => Ok(new_blind_signer(authority_config_path)?),
    }
}

fn load_blind_signer_from_fs(path: &Path) -> Result<blind_sign::BlindSigner> {
    let config = AuthorityConfig::load_from_fs(path)?;
    Ok(blind_sign::BlindSigner::new_from_keys(
        config.pk, config.sk,
    )?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let _tracing_worker_guard = start_logger(&args.data_path.join("authority.log"))?;
    let blind_signer = Arc::new(setup_blind_signer(
        args.new_keys,
        &args.data_path.join("authority-config.json"),
    )?);
    let frontend_path = std::env::current_exe()?
        .parent()
        .ok_or(anyhow!("Could not get parent dir of current executable"))?
        .join("mock_authority_frontend");

    match (args.no_cli, args.no_http_server) {
        (true, true) => bail!("Authority needs at least CLI interface or HTTP server to run"),
        (true, false) => {
            let (_stop_server, handle) = server::run(blind_signer, args.addr, frontend_path);
            handle.await??;
        }
        (false, true) => run_cli(
            &blind_signer,
            args.data_path.join("authority-cmd-history.txt"),
        )?,
        (false, false) => {
            let _server_shutdown = server::run(blind_signer.clone(), args.addr, frontend_path);
            run_cli(
                &blind_signer,
                args.data_path.join("authority-cmd-history.txt"),
            )?;
        }
    }
    Ok(())
}

fn run_cli(blind_signer: &blind_sign::BlindSigner, cmd_history_path: PathBuf) -> Result<()> {
    let mut stdio_reader = StdioReader::new(cmd_history_path)?;

    loop {
        let line = match stdio_reader.read_stdio_blocking() {
            Ok(line) => line,
            Err(e) => {
                println!("Quitting: {e:?}");
                break;
            }
        };
        let res = match Cmd::try_parse_from(line) {
            Ok(Cmd::BlindSign { blinded_msg }) => blind_signer
                .bling_sign(&blinded_msg)
                .map_err(std::convert::Into::into)
                .map(|blinded_signature| blinded_signature.to_string()),
            Ok(Cmd::GetPubkey) => blind_signer
                .get_public_key()
                .map_err(std::convert::Into::into)
                .map(|blinder_pk| blinder_pk.to_string()),
            Ok(Cmd::Quit) => break,
            Err(e) => Err(anyhow!("Unsupported command: {e}")),
        };

        match res {
            Ok(res) => println!("{res}"),
            Err(error) => println!("ERROR: {error}"),
        }
    }

    Ok(())
}
