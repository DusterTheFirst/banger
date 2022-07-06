use std::time::Duration;

use axum::{
    headers::{CacheControl, ContentType, ETag, HeaderMapExt, IfNoneMatch},
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
    Router, TypedHeader,
};
use reqwest::StatusCode;
use rust_embed::RustEmbed;
use tracing::trace;

// TODO: FIXME: Use this (https://docs.rs/tower-http/latest/tower_http/services/struct.ServeDir.html) instead
// with maybe pre-compressed assets that are copied into the dockerfile
pub fn create_router<EmbeddedFiles: RustEmbed>(map_index: bool) -> Router {
    EmbeddedFiles::iter().fold(Router::new(), |mut router, path| {
        let content = EmbeddedFiles::get(&path)
            .expect("RustEmbed content changed during runtime which should be impossible");

        let content_type = mime_guess::from_path(path.as_ref())
            .first()
            .map(ContentType::from);

        let etag = hex::encode(content.metadata.sha256_hash());
        let etag: ETag = format!("\"{etag}\"").parse().unwrap();

        // TODO: must send formatted, not as int
        // if let Some(modified) = content.metadata.last_modified() {
        //     headers.append(header::LAST_MODIFIED, HeaderValue::from(modified))
        // }

        let path = format!("/{path}");

        trace!(?content_type, %path, "registering content");

        let headers = {
            let mut headers = HeaderMap::new();

            if let Some(content_type) = &content_type {
                headers.typed_insert(content_type.clone());
            }
            headers.typed_insert(etag.clone());
            headers.typed_insert(
                CacheControl::new()
                    .with_max_age(Duration::from_secs(21600))
                    .with_no_cache()
                    .with_public(),
            );

            headers
        };

        let service = get(
            move |if_none_match: Option<TypedHeader<IfNoneMatch>>| async move {
                if let Some(TypedHeader(if_none_match)) = if_none_match {
                    if !if_none_match.precondition_passes(&etag) {
                        return (headers, StatusCode::NOT_MODIFIED).into_response();
                    }
                }

                (headers, content.data).into_response()
            },
        );

        if map_index && path == "/index.html" {
            router = router.route("/", service.clone());
            trace!(?content_type, %path, "registering root handler");
        }

        router = router.route(&path, service);

        router
    })
}
