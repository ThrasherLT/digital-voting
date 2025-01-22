//! This is a mock of the election authorities servers which will be responsible for ensuring the
//! eligibility of the voters by signing their public keys. This is only used for testing purposes.

use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use actix_web::{get, post, routes, web, App, HttpResponse, HttpServer, Responder};
use anyhow::{anyhow, bail, Result};
use clap::Parser;
use serde::{self, Deserialize, Serialize};

use crypto::signature::blind_sign;
use process_io::{cli::StdioReader, logging::start_logger};
use tokio::{select, sync::oneshot, task::JoinHandle};
use tracing::trace;

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
    pub data_path: PathBuf,
}

#[derive(Parser, Clone, Debug)]
pub enum Cmd {
    #[clap(about = "Blind sign a blinded message")]
    BlindSign {
        blinded_msg: blind_sign::BlindedMessage,
    },
    #[clap(about = "Get blinder public key")]
    GetPubkey,
    #[clap(about = "Shut down the mock authority")]
    Quit,
}

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

struct AppState {
    blind_signer: Arc<blind_sign::BlindSigner>,
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

    match (args.no_cli, args.no_http_server) {
        (true, true) => bail!("Authority needs at least CLI interface or HTTP server to run"),
        (true, false) => {
            let (_stop_server, handle) = run_server(blind_signer, args.addr);
            handle.await??;
        }
        (false, true) => run_cli(
            &blind_signer,
            args.data_path.join("authority-cmd-history.txt"),
        )?,
        (false, false) => {
            let _server_shutdown = run_server(blind_signer.clone(), args.addr);
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

type Handle = (oneshot::Sender<()>, JoinHandle<Result<(), anyhow::Error>>);

fn run_server(blind_signer: Arc<blind_sign::BlindSigner>, addr: std::net::SocketAddr) -> Handle {
    let (tx, rx) = oneshot::channel::<()>();

    let handle = tokio::spawn(async move {
        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(AppState {
                    blind_signer: blind_signer.clone(),
                }))
                .wrap(
                    actix_cors::Cors::default()
                        // TODO Probably should be more specific:
                        .allow_any_origin()
                        .allow_any_header()
                        .allowed_methods(vec!["GET", "POST"])
                        .max_age(3600),
                )
                .service(greet)
                .service(authenticate)
                .service(get_pkey)
                .service(health)
        })
        .bind(addr)?;
        trace!("Starting server");

        select! {
            _ = rx => {
                Ok(())
            },
            _ = server.run() => {
                bail!("Server stopped")
            }
        }
    });

    (tx, handle)
}

#[routes]
#[get("/")]
#[get("/index.html")]
async fn greet() -> impl Responder {
    trace!("GET root request");
    HttpResponse::Ok().body("Hello, mock authority!\n")
}

#[post("/authenticate")]
pub async fn authenticate(
    verification_request: web::Json<VerificationRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    trace!("POST /authenticate request");
    match data
        .blind_signer
        .bling_sign(&verification_request.blinded_pkey)
    {
        Ok(blind_signature) => HttpResponse::Ok().json(blind_signature),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {e}")),
    }
}

#[get("/pkey")]
pub async fn get_pkey(data: web::Data<AppState>) -> impl Responder {
    trace!("POST /pkey request");
    match data.blind_signer.get_public_key() {
        Ok(pkey) => HttpResponse::Ok().json(pkey.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {e}")),
    }
}

#[get("/health")]
pub async fn health() -> impl Responder {
    trace!("GET /health request");
    HttpResponse::Ok()
}

#[derive(Serialize, Deserialize, Debug)]
struct VerificationRequest {
    blinded_pkey: blind_sign::BlindedMessage,
}
