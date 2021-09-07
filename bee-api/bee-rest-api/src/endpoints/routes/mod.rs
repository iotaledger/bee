pub mod api;

use axum::{
    response::Html,
    handler::get,
    Router,
    routing::BoxRoute
};

pub fn api_routes() -> Router<BoxRoute> {
    Router::new()
        .route("/", get(handler))
        .nest("/api", api::api_routes())
        .boxed()
}


async fn handler() -> Html<&'static str> {
    Html("<h1>You are in /</h1>")
}