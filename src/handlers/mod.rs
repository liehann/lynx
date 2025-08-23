pub mod api;
pub mod ui;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

pub async fn not_found() -> Response {
    (StatusCode::NOT_FOUND, Html("<h1>404 Not Found</h1>")).into_response()
}
