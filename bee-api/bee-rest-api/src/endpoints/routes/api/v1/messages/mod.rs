use bee_message::{Message, MessageId};
use bee_storage::access::Fetch;
use bee_storage_sled::{
    storage::Storage,
};

use axum::{
    body::Body,
    extract::Path,
    handler::{get, post},
    http::Request,
    response::Json,
    Router,
    routing::BoxRoute,
};
use serde_json::{Value, json};
use uuid::Uuid;

pub fn api_routes(storage: &Storage) -> Router<BoxRoute> {
    Router::new()
        .route("/", get(|_req: Request<Body>| async { get_handler(_req, storage) }))
        .route("/", post(post_handler))
        .route("/:messageId", get(|_req: Request<Body>| async {get_id_handler(_req, storage)}))
        .route("/:messageId/metadata", get(get_id_metadata_handler))
        .route("/:messageId/raw", get(get_id_raw_handler))
        .route("/:messageId/children", get(get_id_children_handler))
        .boxed()
}

fn messageId_to_json(messageId: MessageId, storage: &Storage) -> Json<Value> {

    let message = Fetch::fetch(storage, messageId);

    Json(json!({

    }))
}

pub async fn get_handler(req: Request<Body>, storage: &Storage) -> Json<Value> {
    
    
    Json(json!({ 
        "data": {
            "messageId": 555 
        }
    }))
}

pub async fn post_handler() -> Html<&'static str> {
    Html("<h1>You are in /messages with post methode</h1>")
}

async fn get_id_handler(Path(messageId): Path<Uuid>) -> Html<&'static str> {
    Html(format!("<h1>You are in /messages with ID {}</h1>", messageId))
}

pub async fn get_id_metadata_handler(Path(messageId): Path<Uuid>) -> Html<&'static str> {
    Html(format!("<h1>You are in /messages/{}/metadata</h1>", messageId))
}

pub async fn get_id_raw_handler(Path(messageId): Path<Uuid>) -> Html<&'static str> {
    Html(format!("<h1>You are in /messages/{}/raw</h1>", messageId))
}

pub async fn get_id_children_handler(Path(messageId): Path<Uuid>) -> Html<&'static str> {
    Html(format!("<h1>You are in /messages/{}/children</h1>", messageId))
}