pub mod messages;

use axum::{
    Router,
};

pub fn api_routes() -> Router {
    Router::new()
        .nest("/messages", messages::api_routes())
}