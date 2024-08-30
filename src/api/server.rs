use std::net::SocketAddr;

use actix_web::{post, routes, web, App, HttpServer, Responder};
use tracing::info;
use tracing_actix_web::TracingLogger;

use thiserror::Error;

use super::protocol::UnparsedVote;

type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Actix error: {0}")]
    ActixError(#[from] std::io::Error),
}

pub async fn run_server(addr: SocketAddr) -> Result<()> {
    println!("starting HTTP server at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(TracingLogger::default())
            .service(greet)
            .service(vote)
    })
    .bind(addr)?
    .run()
    .await?;

    Ok(())
}

#[routes]
#[get("/")]
#[get("/index.html")]
async fn greet() -> impl Responder {
    "Hello! Please send a POST request to /vote with a JSON body, containing a public key, a vote, a timestamp, and a signature.\n"
}

#[post("/vote")]
pub async fn vote(vote: web::Json<UnparsedVote>) -> impl Responder {
    info!("POST: /vote {vote:?}");
    vote
}
