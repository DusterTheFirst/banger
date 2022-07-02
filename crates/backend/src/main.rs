use std::net::SocketAddr;

use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use tracing::{debug, Level};

use crate::error::not_found;

mod app_content;
mod error;

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main());
}

async fn async_main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let api = Router::new()
        .route("/healthy", get(|| async { "OK" }));

    let app = Router::new()
        .nest("/api", api)
        .fallback(get(app_content::app_content))
        .fallback(get(not_found));

    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));
    debug!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
