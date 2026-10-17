use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode, Uri},
    response::IntoResponse,
};
use axum::response::Response as AxumResponse;
use axum_macros::debug_handler;
use leptos::prelude::LeptosOptions;
use tower::ServiceExt;
use tower_http::services::ServeDir;

#[debug_handler]
pub async fn file_and_error_handler(
    uri: Uri,
    State(options): State<LeptosOptions>,
    _req: Request<Body>,
) -> AxumResponse {
    let root = options.site_root.clone();
    match get_static_file(uri, &root).await {
        Ok(response) => response,
        Err((status, message)) => (status, message).into_response(),
    }
}

async fn get_static_file(
    uri: Uri,
    root: &str,
) -> Result<Response<Body>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(uri)
        .body(Body::empty())
        .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?;
    match ServeDir::new(root).oneshot(req).await {
        Ok(res) => Ok(res.map(Body::new)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {err}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_static_file_returns_not_found() {
        let response = get_static_file(
            Uri::from_static("/favicon.ico"),
            "/path/that/does/not/exist",
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

