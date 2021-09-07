use axum::{
    handler::get,
    response::Html,
    Router,
    routing::BoxRoute
};

pub fn api_routes() -> Router<BoxRoute> {
    Router::new()
        .route("/", get(handler))
        .boxed()
}

pub async fn handler() -> Html<&'static str> {
    Html("<h1>You are in /test</h1>")
}