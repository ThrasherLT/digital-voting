use std::net::SocketAddr;

use actix_cors::Cors;
use actix_web::{get, post, routes, web, App, HttpResponse, HttpServer, Responder};
use anyhow::{bail, Result};
use tokio::{select, sync::oneshot, task::JoinHandle};
use tracing::info;
use tracing_actix_web::TracingLogger;

use protocol::vote::Vote;

use crate::state::State;

pub type Handle = (oneshot::Sender<()>, JoinHandle<Result<(), anyhow::Error>>);

pub fn run(state: State, addr: SocketAddr) -> Result<Handle> {
    let (tx, rx) = oneshot::channel::<()>();
    let state = web::Data::new(state);
    println!("starting HTTP server at {addr}");

    let handle = tokio::spawn(async move {
        let server = HttpServer::new(move || {
            App::new()
                // Actix web takes an app state factory here and uses an Arc internally.
                // It will error in runtime, if state is passed inside an Arc.
                // Also this closure is called once for every worker, meaning that, if you
                // pass State::new(), it'll create a new instance for each worker.
                // https://actix.rs/docs/application#shared-mutable-state
                .app_data(state.clone())
                // Enable logger
                .wrap(TracingLogger::default())
                .wrap(
                    Cors::default()
                        // TODO Probably should be more specific, even though it's a browser extension:
                        .allow_any_origin()
                        .allow_any_header()
                        .allowed_methods(vec!["GET", "POST"])
                        .max_age(3600),
                )
                .service(greet)
                .service(vote)
                .service(config)
                .service(health)
        })
        .bind(addr)?;

        select! {
            _ = rx => {
                Ok(())
            },
            _ = server.run() => {
                bail!("Server stopped")
            }
        }
    });

    Ok((tx, handle))
}

#[routes]
#[get("/")]
#[get("/index.html")]
async fn greet() -> impl Responder {
    HttpResponse::Ok().body("Welcome to the digital voting blockchain! (WIP).\n")
}

#[post("/vote")]
pub async fn vote(vote: web::Json<Vote>) -> impl Responder {
    info!("POST: /vote {vote:?}");
    HttpResponse::Ok()
}

#[get("/config")]
pub async fn config(state: web::Data<State>) -> impl Responder {
    if let Ok(json) = serde_json::to_string(state.get_blockchain_config()) {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::InternalServerError().body("Failed to get config")
    }
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok()
}
