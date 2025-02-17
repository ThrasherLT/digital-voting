use std::{path::PathBuf, sync::Arc};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use tokio::{select, sync::oneshot, task::JoinHandle};
use tracing::trace;

use crypto::signature::blind_sign;

pub type Handle = (oneshot::Sender<()>, JoinHandle<Result<(), anyhow::Error>>);

struct AppState {
    blind_signer: Arc<blind_sign::BlindSigner>,
}

pub fn run(
    blind_signer: Arc<blind_sign::BlindSigner>,
    addr: std::net::SocketAddr,
    frontend_path: PathBuf,
) -> Handle {
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
                .service(authenticate)
                .service(get_pkey)
                .service(health)
                // For some reason the files service must be last in this call list, otherwise, the services after it
                // won't work.
                .service(
                    actix_files::Files::new("/", frontend_path.clone()).index_file("index.html"),
                )
        })
        .bind(addr)?;
        trace!("Starting server");

        select! {
            _ = rx => {
                Ok(())
            },
            _ = server.run() => {
                anyhow::bail!("Server stopped")
            }
        }
    });

    (tx, handle)
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct VerificationRequest {
    blinded_pkey: blind_sign::BlindedMessage,
}
