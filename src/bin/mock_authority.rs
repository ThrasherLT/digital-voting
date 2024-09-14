//! This is a mock of the election authorities servers which will be responsible for ensuring the
//! eligibility of the voters by signing their public keys. This is only used for testing purposes.

// TODO test this out.

use actix_web::{get, post, routes, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use clap::Parser;
use crypto::signature::blind_signature::{BlindSignature, BlindSigner, BlindedMessage};
use digital_voting::json_base64::json_base64_ser;
use digital_voting::logging::start_logger;
use digital_voting::Timestamp;
use serde::{self, Deserialize, Serialize};
use tracing::info;

#[derive(Parser, Clone, Debug)]
pub struct Args {
    #[clap(short = 'a', long = "address", default_value = "127.0.0.1:8081")]
    pub addr: std::net::SocketAddr,
}

struct AppState {
    blind_signer: BlindSigner,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _tracing_worker_guard = start_logger("mock_authority.log")?;
    let args = Args::parse();

    println!("Starting mock authority on: {}...", args.addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                blind_signer: BlindSigner::new().expect("Could not create a blind signer"),
            }))
            .service(greet)
            .service(verify)
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
    "Hello! Please send a POST request to /verify with a JSON body, containing a public key, a vote, some mock authentication data, and a signature.\n"
}

#[post("/verify")]
pub async fn verify(
    verification_request: web::Json<VerificationRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    info!("POST: /vote {verification_request:?}");
    if mock_verify_authentication_data(verification_request.authentication_data.as_str()) {
        match data
            .blind_signer
            .bling_sign(BlindedMessage(verification_request.pkey.clone()))
        {
            Ok(blind_signature) => {
                HttpResponse::Ok().json(VerificationResponse::Verified { blind_signature })
            }
            Err(e) => HttpResponse::InternalServerError().body(format!("Error: {e}")),
        }
    } else {
        HttpResponse::BadRequest().json(VerificationResponse::Denied)
    }
}

#[get("/pkey")]
pub async fn get_pkey(data: web::Data<AppState>) -> impl Responder {
    match data.blind_signer.get_public_key() {
        Ok(pkey) => HttpResponse::Ok().json(pkey),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {e}")),
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct VerificationRequest {
    #[serde(with = "json_base64_ser")]
    pkey: Vec<u8>,
    authentication_data: String,
    timestamp: Timestamp,
    #[serde(with = "json_base64_ser")]
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
enum VerificationResponse {
    Verified { blind_signature: BlindSignature },
    Denied,
}

fn mock_verify_authentication_data(authentication_data: &str) -> bool {
    !authentication_data.is_empty()
}
