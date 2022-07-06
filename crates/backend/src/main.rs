use std::{env, net::SocketAddr};

use axum::{routing::get, Router};
use rust_embed::RustEmbed;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, metrics::InFlightRequestsLayer, ServiceBuilderExt};
use tracing::{debug, error};
use tracing_subscriber::EnvFilter;

use crate::error::not_found;

mod api;
mod error;
mod serde;
mod static_content;

fn main() {
    #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_error| {
            EnvFilter::new("info,spotify_banger_backend=trace,spotify_banger_model=trace")
        }))
        .init();

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
    // TODO: add counter to prometheus
    let (in_flight_layer, in_flight_counter) = InFlightRequestsLayer::pair();

    let app = Router::new()
        .merge(static_content::create_router::<WebAppContent>(true))
        .fallback(get(not_found))
        .layer(CorsLayer::permissive())
        .nest("/api", api::create_router())
        .layer(
            ServiceBuilder::new()
                .trace_for_http()
                .compression()
                .layer(in_flight_layer),
        );

    let addr = env::var("BIND")
        .ok()
        .and_then(|addr| {
            addr.parse()
                .map_err(|error| {
                    error!(%error, "failed to parse BIND environment variable");
                    error
                })
                .ok()
        })
        .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 9000)));

    debug!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
