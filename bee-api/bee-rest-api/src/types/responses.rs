// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{body::BodyInner, dtos::MessageDto};

use serde::{Deserialize, Serialize};

/// Response of POST /api/v1/messages.
/// Returns the message identifier of the submitted message.
#[derive(Clone, Debug, Serialize)]
pub struct SubmitMessageResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
}

impl BodyInner for SubmitMessageResponse {}

/// Response of GET /api/v1/messages?index={INDEX}.
/// Returns all messages ids that match a given indexation key.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessagesFindResponse {
    pub index: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    #[serde(rename = "messageIds")]
    pub message_ids: Vec<String>,
}

impl BodyInner for MessagesFindResponse {}

/// Response of GET /api/v1/messages/{message_id}.
/// Returns a specific message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageResponse(pub MessageDto);

impl BodyInner for MessageResponse {}

/// Response of GET /api/v1/messages/{message_id}/metadata.
/// Returns the metadata of a message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageMetadataResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "parentMessageIds")]
    pub parent_message_ids: Vec<String>,
}

impl BodyInner for MessageMetadataResponse {}

/// Response of GET /api/v1/messages/{message_id}/children.
/// Returns all children of a specific message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageChildrenResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    #[serde(rename = "childrenMessageIds")]
    pub children_message_ids: Vec<String>,
}

impl BodyInner for MessageChildrenResponse {}
