// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    message::{self, Message, MESSAGE_PUBLIC_KEY_LENGTH, MESSAGE_SIGNATURE_LENGTH},
    parents::ParentsBlock,
    payload::Payload,
    ValidationError,
};

use bee_packable::Packable;

use alloc::vec::Vec;

/// A builder to build a [`Message`].
#[derive(Default)]
pub struct MessageBuilder {
    parents_blocks: Vec<ParentsBlock>,
    issuer_public_key: Option<[u8; MESSAGE_PUBLIC_KEY_LENGTH]>,
    issue_timestamp: Option<u64>,
    sequence_number: Option<u32>,
    payload: Option<Payload>,
    nonce: Option<u64>,
    signature: Option<[u8; MESSAGE_SIGNATURE_LENGTH]>,
}

impl MessageBuilder {
    /// Creates a new [`MessageBuilder`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a [`ParentsBlock`] to a [`MessageBuilder`].
    pub fn add_parents_block(mut self, parents_block: ParentsBlock) -> Self {
        self.parents_blocks.push(parents_block);
        self
    }

    /// Adds an issuer public key to a [`MessageBuilder`].
    pub fn with_issuer_public_key(mut self, issuer_public_key: [u8; MESSAGE_PUBLIC_KEY_LENGTH]) -> Self {
        self.issuer_public_key.replace(issuer_public_key);
        self
    }

    /// Adds an issuance timestamp to a [`MessageBuilder`].
    pub fn with_issue_timestamp(mut self, issue_timestamp: u64) -> Self {
        self.issue_timestamp.replace(issue_timestamp);
        self
    }

    /// Adds a sequence number to a [`MessageBuilder`].
    pub fn with_sequence_number(mut self, sequence_number: u32) -> Self {
        self.sequence_number.replace(sequence_number);
        self
    }

    /// Adds a payload to a [`MessageBuilder`].
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload.replace(payload);
        self
    }

    /// Adds a nonce provider to a [`MessageBuilder`].
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce.replace(nonce);
        self
    }

    /// Adds a signature to a [`MessageBuilder`].
    pub fn with_signature(mut self, signature: [u8; MESSAGE_SIGNATURE_LENGTH]) -> Self {
        self.signature.replace(signature);
        self
    }

    /// Finishes the [`MessageBuilder`], consuming it to build a [`Message`].
    pub fn finish(self) -> Result<Message, ValidationError> {
        message::validate_parents_blocks_count(self.parents_blocks.len())?;
        message::validate_has_strong_parents(&self.parents_blocks)?;

        let issuer_public_key = self
            .issuer_public_key
            .ok_or(ValidationError::MissingField("issuer_public_key"))?;
        let issue_timestamp = self
            .issue_timestamp
            .ok_or(ValidationError::MissingField("issue_timestap"))?;
        let sequence_number = self
            .sequence_number
            .ok_or(ValidationError::MissingField("sequence_number"))?;

        let nonce = self.nonce.ok_or(ValidationError::MissingField("nonce"))?;
        let signature = self.signature.ok_or(ValidationError::MissingField("signature"))?;

        let message = Message {
            parents_blocks: self.parents_blocks,
            issuer_public_key,
            issue_timestamp,
            sequence_number,
            payload: self.payload,
            nonce,
            signature,
        };

        // This unwrap is fine, because we have just unpacked a valid message.
        message::validate_message_len(message.pack_to_vec().unwrap().len())?;

        Ok(message)
    }
}
