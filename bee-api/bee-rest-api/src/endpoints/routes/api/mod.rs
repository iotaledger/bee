pub mod v1;

use bee_storage_sled::storage::Storage;

use axum::{
    handler::get,
    Router,
    response::Html,
    routing::BoxRoute
};

pub fn api_routes(storage: &Storage) -> Router<BoxRoute> {
    Router::new()
        .route("/", get(handler))
        .nest("/v1", v1::api_routes(storage))
        .boxed()
}

async fn handler() -> Html<&'static str> {
    Html("<h1>You are in /api</h1>")
}