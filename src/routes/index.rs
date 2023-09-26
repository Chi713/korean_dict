use axum::routing::get;
use axum::Router;
use axum::response::{Html, IntoResponse};
use axum::Extension;
use tera::Context;
use tera::Tera;
use std::sync::Arc;

pub fn index() -> Router {
    async fn handler(
        Extension(templates): Extension<Arc<Tera>>,
    ) -> impl IntoResponse {
        let context = Context::new();
        Html(templates.render("index.html", &context).unwrap())
    }
    Router::new().route("/", get(|template: Extension<Arc<Tera>>| handler(template)))
}

