use axum::{
    extract::Path,
    handler::{get, post},
    response::Html,
    Router,
    routing::BoxRoute
};
use uuid::Uuid;

pub fn api_routes() -> Router<BoxRoute> {
    Router::new()
        .route("/", get(get_handler))
        .route("/", post(post_handler))
        .route("/:messageId", get(get_id_handler))
        .route("/:messageId/metadata", get(get_id_metadata_handler))
        .route("/:messageId/raw", get(get_id_raw_handler))
        .route("/:messageId/children", get(get_id_children_handler))
        .boxed()
}

pub async fn get_handler() -> Html<&'static str> {
    Html("<h1>You are in /messages with get methode</h1>")
}

pub async fn post_handler() -> Html<&'static str> {
    Html("<h1>You are in /messages with post methode</h1>")
}

async fn get_id_handler(Path(messageId): Path<Uuid>) -> Html<&'static str> {
    Html("<h1>You are in /messages with get methode</h1>")
}

pub async fn get_id_metadata_handler() -> Html<&'static str> {
    Html("<h1>You are in /messages with get methode</h1>")
}

pub async fn get_id_raw_handler() -> Html<&'static str> {
    Html("<h1>You are in /messages with get methode</h1>")
}

pub async fn get_id_children_handler() -> Html<&'static str> {
    Html("<h1>You are in /messages with get methode</h1>")
}