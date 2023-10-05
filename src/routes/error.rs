use log::error;
use thiserror::Error;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Error, Debug)]
pub enum RouteError {
    #[error("transparent")]
    SqlxError(#[from] sqlx::Error),
    #[error("failed to get last csv row error")]
    LastCsvRowError,
    #[error("page not found")]
    PageNotFound,
    #[error("transparent")]
    Unknown(#[from] anyhow::Error),
}

impl IntoResponse for RouteError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            RouteError::SqlxError(source) => {
                error!("{}", source);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("the was an error retrieving data from database"))
            }
            RouteError::LastCsvRowError => {
                error!("failed to get last csv row error");
                (StatusCode::INTERNAL_SERVER_ERROR, format!("failed to get last csv row error"))
            }
            RouteError::PageNotFound => {
                error!("failed to get last csv row error");
                (StatusCode::NOT_FOUND, format!("failed to get last csv row error"))
            }
            RouteError::Unknown(source) => {
                error!("{}", source);
                (StatusCode::INTERNAL_SERVER_ERROR, "Unknown Internal Error".into())
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
