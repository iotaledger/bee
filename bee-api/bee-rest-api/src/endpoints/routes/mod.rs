pub mod api;

use bee_storage_sled::storage::Storage;

use axum::{
    response::Html,
    handler::get,
    Router,
    routing::BoxRoute
};

pub fn api_routes(storage: &Storage) -> Router<BoxRoute> {
    Router::new()
        .route("/", get(handler))
        .nest("/api", api::api_routes(storage))
        .boxed()
}


async fn handler() -> Html<&'static str> {
    Html("<h1>You are in /</h1>")
}