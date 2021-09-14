use axum::{
    extract::Path,
    handler::get,
    Router,
};
use uuid::Uuid;

async fn user_info(Path(user_id): Path<Uuid>) {
    // ...
}
fn test() -> int {
    let app = Router::new().route("/users/:user_id", get(user_info));
}