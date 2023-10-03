use thiserror::Error;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Error, Debug)]
pub enum RouteError {
    #[error("sqlx error")]
    RowNotFound(#[from] sqlx::Error),

    #[error("Unknown Internal Error")]
    Internal
}

impl IntoResponse for RouteError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            RouteError::RowNotFound(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "the was an error retrieving data from database")
            }
            RouteError::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unknown Internal Error")
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
