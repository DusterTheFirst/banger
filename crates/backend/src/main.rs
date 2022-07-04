use std::net::SocketAddr;

use axum::{routing::get, Router};
use rust_embed::RustEmbed;
use tracing::debug;
use tracing_subscriber::EnvFilter;

use crate::error::not_found;

mod static_content;
mod error;

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main());
}


#[derive(RustEmbed)]
#[folder = "../client/dist/"]
struct WebAppContent;

async fn async_main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_error| {
            EnvFilter::new("info,spotify_banger_backend=trace,spotify_banger_model=trace")
        }))
        .init();

    let api = Router::new().route("/healthy", get(|| async { "OK" }));

    let app = Router::new()
        .nest("/api", api)
        .merge(static_content::create_router::<WebAppContent>(true))
        .fallback(get(not_found));

    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));
    debug!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
