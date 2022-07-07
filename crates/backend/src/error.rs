use askama::Template;
use axum::{
    body::Body,
    http::{self, request, Request, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use tracing::{error, trace};

macro_rules! derive_into_response {
    ($ty:ty) => {
        impl IntoResponse for $ty {
            fn into_response(self) -> axum::response::Response {
                into_response(self)
            }
        }
    };
}

pub fn into_response<T: Template>(t: T) -> Response {
    match t.render() {
        Ok(body) => {
            let headers = [(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static(T::MIME_TYPE),
            )];

            (headers, body).into_response()
        }
        Err(error) => {
            error!(%error, "askama encountered an error while rendering");

            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Debug, Template)]
#[template(path = "404.html")]
pub struct NotFound {
    path: String,
}

derive_into_response!(NotFound);

pub async fn not_found<E>(request: Request<Body>) -> Result<Response, E> {
    let path = request.uri().path();

    trace!(path, "user requested unknown path");

    Ok((
        StatusCode::NOT_FOUND,
        NotFound {
            path: path.to_string(),
        },
    )
        .into_response())
}
