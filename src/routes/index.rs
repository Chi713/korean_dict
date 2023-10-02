use axum::routing::get;
use axum::Router;
use axum::response::IntoResponse;
use axum::http::StatusCode;
use super::templates::IndexTemplate;

pub fn index() -> Router {
    async fn handler() -> Result<impl IntoResponse, (StatusCode, String)> {
        let s = IndexTemplate{};
        Ok(s)
    }
    Router::new().route("/", get(handler))
}

