use std::{
    env, io,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use axum::{
    routing::{any_service, get, get_service},
    Router,
};
use reqwest::StatusCode;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer, metrics::InFlightRequestsLayer, services::ServeDir, ServiceBuilderExt,
};
use tracing::{debug, error, Level};
use tracing_subscriber::EnvFilter;

use crate::error::not_found;

mod api;
mod error;
mod serde;

fn main() {
    #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_error| {
            EnvFilter::default()
                .add_directive(Level::INFO.into())
                .add_directive("tower_http=debug".parse().unwrap())
                .add_directive("spotify_banger_backend=trace".parse().unwrap())
                .add_directive("spotify_banger_model=trace".parse().unwrap())
        }))
        .init();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main());
}

async fn async_main() {
    // TODO: add counter to prometheus
    let (in_flight_layer, in_flight_counter) = InFlightRequestsLayer::pair();

    let app = Router::new()
        .fallback(
            any_service(
                ServeDir::new(env::var("STATIC_FILES").unwrap_or_else(|_| {
                    format!(
                        "{}/../client/dist",
                        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set")
                    )
                }))
                .precompressed_br()
                .append_index_html_on_directories(true)
                .fallback(get_service(tower::service_fn(not_found::<io::Error>))),
            )
            .handle_error(|_err: io::Error| async {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
            }),
        )
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
