use axum::{
    body::{boxed, Empty, Full},
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../client/dist/"]
struct AppContent;

pub async fn app_content(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    AppContentResponse {
        path: if path.is_empty() { "index.html" } else { path }.to_string(),
    }
}

pub struct AppContentResponse {
    path: String,
}

impl IntoResponse for AppContentResponse {
    fn into_response(self) -> Response {
        match AppContent::get(self.path.as_str()) {
            Some(content) => {
                let mut builder = Response::builder();

                if let Some(mime) = mime_guess::from_path(self.path).first() {
                    builder = builder.header(header::CONTENT_TYPE, mime.as_ref());
                }

                builder.body(boxed(Full::from(content.data))).unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(boxed(Empty::new()))
                .unwrap(),
        }
    }
}
