use axum::{
    http::{header, HeaderMap, HeaderValue},
    response::IntoResponse,
    routing::get,
    Router,
};
use rust_embed::RustEmbed;
use tracing::trace;

pub fn create_router<EmbeddedFiles: RustEmbed>(map_index: bool) -> Router {
    EmbeddedFiles::iter().fold(Router::new(), |mut router, path| {
        let mime = mime_guess::from_path(path.as_ref()).first();

        let content = EmbeddedFiles::get(&path)
            .expect("RustEmbed content changed during runtime which should be impossible");

        let mut headers = HeaderMap::new();

        if let Some(mime) = &mime {
            headers.append(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime.as_ref()).unwrap(),
            );
        }

        headers.append(
            header::ETAG,
            HeaderValue::from_str(&hex::encode(content.metadata.sha256_hash())).unwrap(),
        );

        // TODO: must send formatted, not as int
        // if let Some(modified) = content.metadata.last_modified() {
        //     headers.append(header::LAST_MODIFIED, HeaderValue::from(modified))
        // }

        let path = format!("/{path}");

        trace!(?mime, %path, "registering content");

        let service = get(move || async move { (headers, content.data).into_response() });

        if map_index && path == "/index.html" {
            router = router.route("/", service.clone());
            trace!(?mime, %path, "registering root handler");
        }

        router = router.route(&path, service);

        router
    })
}
