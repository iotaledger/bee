pub mod endpoints;

use bee_storage_sled::{
    storage::Storage,
    config::SledConfigBuilder,
};

use axum::{handler::get, response::Html, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let sled_config = SledConfigBuilder::new().finish();
    let storage;

    match Storage::new(sled_config) {
        Err(e) => println!("Error creating storage config {:?}", e),
        Ok(conf) => storage = conf,
    }
    // build our application with a route
    let app = Router::new()
        .route("/", get(handler))
        .nest("/api", endpoints::routes::api::api_routes(&storage));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
