// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod endpoints;
pub mod types;

use bee_storage_sled::{config::SledConfigBuilder, storage::Storage};

use axum::{AddExtensionLayer, Router};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

pub struct AppStorage {
    storage: Mutex<Storage>,
}

#[tokio::main]
async fn main() {
    let sled_config = SledConfigBuilder::new().finish();
    let storage;

    match Storage::new(sled_config) {
        Err(e) => {
            println!("Error creating storage config {:?}", e);
            return;
        }
        Ok(conf) => storage = conf,
    }
    let app_storage = Arc::new(AppStorage {
        storage: Mutex::new(storage),
    });
    let app = Router::new()
        .nest("/api", endpoints::routes::api::api_routes())
        .layer(AddExtensionLayer::new(app_storage));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}
