// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::AppStorage;
use crate::types::{
    dtos::MessageDto,
    body::{SuccessBody, ErrorBody, DefaultErrorResponse},
    responses::{SubmitMessageResponse, MessageResponse, MessageMetadataResponse, MessageChildrenResponse},
};

use bee_message::{Message, MessageId};
use bee_storage::access::{Fetch, Insert};

use axum::{
    extract::{Path, Extension},
    routing::{get, post},
    response::Json,
    Router,
};
use serde_json::{Value, json};

use std::{
    sync::Arc,
    str::FromStr,
};
use std::convert::{TryFrom};
use rand::Rng;

pub fn api_routes() -> Router {
    Router::new()
        .route("/", post(post_handler))
        .route("/:messageId", get(get_id_handler))
        .route("/:messageId/metadata", get(get_id_metadata_handler))
        .route("/:messageId/raw", get(get_id_raw_handler))
        .route("/:messageId/children", get(get_id_children_handler))
}

pub async fn post_handler(Extension(app_storage): Extension<Arc<AppStorage>>, Json(payload): Json<MessageDto>) -> Json<Value> {
    let mut rng = rand::thread_rng();

    let rand_part1: u128 = rng.gen();
    let rand_part2: u128 = rng.gen();
    let rand_id = format!("{:x}{:x}", rand_part1, rand_part2);
    match Insert::<MessageId, Message>::insert(&*(
        app_storage.storage.lock().unwrap()),
        match &MessageId::from_str(&rand_id) {
            Ok(message_id) => message_id,
            Err(e) => return Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could not parse message ID. {}", e)}))),
        },
        &match Message::try_from(&payload) {
            Ok(message) => message,
            Err(e) => return Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could not parse message data. {}", e)}))),
        }
    ) {
        Ok(()) => Json(json!(SuccessBody::new(SubmitMessageResponse{message_id: rand_id}))),
        Err(e) => Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could store message. {}", e)}))),
    }
}

async fn get_id_handler(Path(messageid): Path<String>, Extension(app_storage): Extension<Arc<AppStorage>>) -> Json<Value> {

    match Fetch::<MessageId, Message>::fetch(&*(
        app_storage.storage.lock().unwrap()),
            match &MessageId::from_str(&messageid.to_string()) {
                Ok(message_id) => message_id,
                Err(e) => return Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could not parse message ID. {}", e)}))),
            }
        ) {
            Ok(message) => match message {
                Some(message) => Json(match serde_json::to_value(&SuccessBody::new(MessageResponse(MessageDto::from(&message)))) {
                    Ok(data) => data,
                    Err(e) => json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could not convert message. {}", e)})),
                }),
                None => Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could find message.")}))),
                }
            Err(e) => Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could not fetch message from storage. {}", e)}))),
    }
}

pub async fn get_id_metadata_handler(Path(messageid): Path<String>, Extension(app_storage): Extension<Arc<AppStorage>>) -> Json<Value> {
    match Fetch::<MessageId, Message>::fetch(&*(
        app_storage.storage.lock().unwrap()),
            match &MessageId::from_str(&messageid.to_string()) {
                Ok(message_id) => message_id,
                Err(e) => return Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could not parse message ID. {}", e)}))),
            }
        ) {
            Ok(message) => match message {
                Some(message) => Json(match serde_json::to_value(&SuccessBody::new( {
                    let message_dto = MessageDto::from(&message);
                    MessageMetadataResponse {
                        message_id: message.id().to_string(),
                        parent_message_ids: message_dto.parents.iter().map(|p| p.message_id.clone()).collect(),
                    }
                })) {
                    Ok(data) => data,
                    Err(e) => json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could not convert message. {}", e)})),
                }),
                None => Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could find message.")}))),
                }
            Err(e) => Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could not fetch message from storage. {}", e)}))),
    }
}

pub async fn get_id_raw_handler(Path(messageid): Path<String>, Extension(app_storage): Extension<Arc<AppStorage>>) -> Json<Value> {
    match Fetch::<MessageId, Message>::fetch(&*(
        app_storage.storage.lock().unwrap()),
            match &MessageId::from_str(&messageid.to_string()) {
                Ok(message_id) => message_id,
                Err(e) => return Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could not parse message ID. {}", e)}))),
            }
        ) {
            Ok(message) => match message {
                Some(message) => Json(match serde_json::to_value(&SuccessBody::new( {
                    let message_dto = MessageDto::from(&message);
                    MessageMetadataResponse { // TODO - change to Raw message
                        message_id: message.id().to_string(),
                        parent_message_ids: message_dto.parents.iter().map(|p| p.message_id.clone()).collect(),
                    }
                })) {
                    Ok(data) => data,
                    Err(e) => json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could not convert message. {}", e)})),
                }),
                None => Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could find message.")}))),
                }
            Err(e) => Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could not fetch message from storage. {}", e)}))),
    }
}

pub async fn get_id_children_handler(Path(messageid): Path<String>, Extension(app_storage): Extension<Arc<AppStorage>>) -> Json<Value> {
    match Fetch::<MessageId, Message>::fetch(&*(
        app_storage.storage.lock().unwrap()),
            match &MessageId::from_str(&messageid.to_string()) {
                Ok(message_id) => message_id,
                Err(e) => return Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could not parse message ID. {}", e)}))),
            }
        ) {
            Ok(message) => match message {
                Some(message) => Json(match serde_json::to_value(&SuccessBody::new( {
                    let _message_dto = MessageDto::from(&message);
                    MessageChildrenResponse {
                        message_id: message.id().to_string(),
                        max_results: 0, // TODO
                        count: 0, // TODO
                        children_message_ids: vec!(), // TODO
                    }
                })) {
                    Ok(data) => data,
                    Err(e) => json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could not convert message. {}", e)})),
                }),
                None => Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could find message.")}))),
                }
            Err(e) => Json(json!(ErrorBody::new(DefaultErrorResponse{code: "400".to_string(), message: format!("could not fetch message from storage. {}", e)}))),
    }
}