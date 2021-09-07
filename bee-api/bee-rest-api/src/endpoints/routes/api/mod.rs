pub mod v1;

use axum::{
    handler::get,
    Router,
    response::Html,
    routing::BoxRoute
};

pub fn api_routes() -> Router<BoxRoute> {
    Router::new()
        .route("/", get(handler))
        .nest("/v1", v1::api_routes())
        .boxed()
}

async fn handler() -> Html<&'static str> {
    Html("<h1>You are in /api</h1>")
}