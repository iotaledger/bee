pub mod api;

use axum::{
    Router,
};

pub fn api_routes() -> Router {
    Router::new()
        .nest("/api", api::api_routes())
}