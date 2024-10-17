//! This is a mock of the election authorities servers which will be responsible for ensuring the
//! eligibility of the voters by signing their public keys. This is only used for testing purposes.

use std::io::Write;

use actix_web::{get, post, routes, web, App, HttpResponse, HttpServer, Responder};
use anyhow::{anyhow, Result};
use clap::Parser;
use crypto::signature::blind_sign::{self, BlindSigner};
use digital_voting::logging::start_logger;
use serde::{self, Deserialize, Serialize};

#[derive(Parser, Clone, Debug)]
pub struct Args {
    #[clap(short = 'a', long = "address", default_value = "127.0.0.1:8081")]
    pub addr: std::net::SocketAddr,
    #[clap(short = 'k', long = "new-keys", default_value_t = false)]
    pub new_keys: bool,
}

struct AppState {
    blind_signer: BlindSigner,
}

fn new_blind_signer(path: &str) -> Result<BlindSigner> {
    let blind_signer = BlindSigner::new()?;
    let mut blind_signer_cfg_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    writeln!(blind_signer_cfg_file, "{}", blind_signer.get_public_key()?)?;
    writeln!(blind_signer_cfg_file, "{}", blind_signer.get_secret_key()?)?;

    Ok(blind_signer)
}

fn setup_blind_signer(arg_new_keys: bool) -> Result<BlindSigner> {
    let blind_signer_cfg_path = "authority-blind-signer-cfg";

    if arg_new_keys {
        if let Err(err) = std::fs::remove_file(blind_signer_cfg_path) {
            // It's not an error if the file actually doesn't exist, since we're deleting it anyway.
            if err.kind() != std::io::ErrorKind::NotFound {
                anyhow::bail!("Failed to delete old blind signer cfg file: {}", err);
            }
        }
    }

    match load_blind_signer_from_fs(blind_signer_cfg_path) {
        Ok(blind_signer) => Ok(blind_signer),
        Err(_) => Ok(new_blind_signer(blind_signer_cfg_path)?),
    }
}

fn load_blind_signer_from_fs(path: &str) -> Result<BlindSigner> {
    if std::path::Path::new(path).exists() {
        let blind_signer_cfg = std::fs::read_to_string(path)?;
        let mut blind_signer_cfg = blind_signer_cfg.lines().take(2);
        let (pk, sk) = (
            blind_signer_cfg
                .next()
                .ok_or(anyhow!("Failed to parse blind signer public key"))?
                .parse()?,
            blind_signer_cfg
                .next()
                .ok_or(anyhow!("Failed to parse blind signer secret key"))?
                .parse()?,
        );
        Ok(BlindSigner::new_from_keys(pk, sk)?)
    } else {
        Err(anyhow!("Blind signer config not found"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _tracing_worker_guard = start_logger("mock_authority.log")?;
    let args = Args::parse();

    println!("Starting mock authority server on: {}...", args.addr);

    let blind_signer = setup_blind_signer(args.new_keys)?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                // TODO expect:
                blind_signer: blind_signer.clone(),
            }))
            .service(greet)
            .service(authenticate)
            .service(get_pkey)
    })
    .bind(args.addr)?
    .run()
    .await?;

    Ok(())
}

#[routes]
#[get("/")]
#[get("/index.html")]
async fn greet() -> impl Responder {
    "Hello! Please send a POST request to /authenticate with a JSON body, containing a public key, a vote, some mock authentication data, and a signature.\n"
}

#[post("/authenticate")]
pub async fn authenticate(
    verification_request: web::Json<VerificationRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
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
    match data.blind_signer.get_public_key() {
        Ok(pkey) => HttpResponse::Ok().json(pkey.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {e}")),
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct VerificationRequest {
    blinded_pkey: blind_sign::BlindedMessage,
}
