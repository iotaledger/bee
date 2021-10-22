// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use num::CheckedAdd;

use crate::{
    delay::{DelayFactory, Repeat},
    discovery_messages::{DiscoveryRequest, VerificationRequest, VerificationResponse},
    hash,
    identity::PeerId,
    local::Local,
    packet::{msg_hash, MessageType},
    peering_messages::PeeringRequest,
    salt::Salt,
    shutdown::ShutdownRx,
    time::{self, Timestamp},
};

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    net::{IpAddr, SocketAddr},
    ops::DerefMut,
    sync::{Arc, RwLock},
    time::Duration,
};

type RequestHash = [u8; hash::SHA256_LEN];

// If the request is not answered within that time it gets removed from the manager, and any response
// coming in later will be deemed invalid.
pub(crate) const REQUEST_EXPIRATION_SECS: u64 = 20;

// Marker trait for requests.
pub(crate) trait Request: Debug + Clone {}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) struct RequestKey {
    pub(crate) peer_id: PeerId,
    pub(crate) request_id: TypeId,
}

pub(crate) struct RequestValue {
    pub(crate) request_hash: [u8; hash::SHA256_LEN],
    pub(crate) expiration_time: u64,
}

#[derive(Clone)]
pub(crate) struct RequestManager {
    pub(crate) version: u32,
    pub(crate) network_id: u32,
    pub(crate) source_addr: SocketAddr,
    pub(crate) local: Local,
    pub(crate) open_requests: Arc<RwLock<HashMap<RequestKey, RequestValue>>>,
}

impl RequestManager {
    pub(crate) fn new(version: u32, network_id: u32, source_addr: SocketAddr, local: Local) -> Self {
        Self {
            version,
            network_id,
            source_addr,
            local,
            open_requests: Arc::new(RwLock::new(HashMap::default())),
        }
    }

    pub(crate) fn new_verification_request(&self, peer_id: PeerId, target_addr: IpAddr) -> VerificationRequest {
        let timestamp = crate::time::unix_now_secs();

        let key = RequestKey {
            peer_id,
            request_id: TypeId::of::<VerificationRequest>(),
        };

        let verif_req = VerificationRequest {
            version: self.version,
            network_id: self.network_id,
            timestamp,
            source_addr: self.source_addr,
            target_addr,
        };

        let request_hash = msg_hash(
            MessageType::VerificationRequest,
            &verif_req.to_protobuf().expect("error encoding verification request"),
        );

        let value = RequestValue {
            request_hash,
            expiration_time: timestamp + REQUEST_EXPIRATION_SECS,
        };

        let _ = self
            .open_requests
            .write()
            .expect("error getting write access")
            .insert(key, value);

        verif_req
    }

    pub(crate) fn new_discovery_request(&self, peer_id: PeerId, target_addr: IpAddr) -> DiscoveryRequest {
        let timestamp = crate::time::unix_now_secs();

        let key = RequestKey {
            peer_id,
            request_id: TypeId::of::<DiscoveryRequest>(),
        };

        let disc_req = DiscoveryRequest { timestamp };

        let request_hash = msg_hash(
            MessageType::DiscoveryRequest,
            &disc_req.to_protobuf().expect("error encoding discovery request"),
        );

        let value = RequestValue {
            request_hash,
            expiration_time: timestamp + REQUEST_EXPIRATION_SECS,
        };

        let _ = self
            .open_requests
            .write()
            .expect("error getting write access")
            .insert(key, value);

        disc_req
    }

    pub(crate) fn new_peering_request(&self, peer_id: PeerId) -> PeeringRequest {
        let timestamp = crate::time::unix_now_secs();

        let key = RequestKey {
            peer_id,
            request_id: TypeId::of::<PeeringRequest>(),
        };

        let peer_req = PeeringRequest {
            timestamp,
            salt: self.local.public_salt().expect("missing public salt"),
        };

        let request_hash = msg_hash(
            MessageType::PeeringRequest,
            &peer_req.to_protobuf().expect("error encoding peering request"),
        );

        let value = RequestValue {
            request_hash,
            expiration_time: timestamp + REQUEST_EXPIRATION_SECS,
        };

        let _ = self
            .open_requests
            .write()
            .expect("error getting write access")
            .insert(key, value);

        peer_req
    }

    pub(crate) fn get_request_hash<R: Request + 'static>(&self, peer_id: &PeerId) -> Option<RequestHash> {
        // TODO: Can we prevent the clone?
        let key = RequestKey {
            peer_id: peer_id.clone(),
            request_id: TypeId::of::<R>(),
        };

        let requests = self.open_requests.read().expect("error getting read access");
        if let Some(RequestValue { request_hash, .. }) = (*requests).get(&key) {
            Some(request_hash.clone())
        } else {
            None
        }
    }

    pub(crate) fn remove_request<R: Request + 'static>(&self, peer_id: &PeerId) -> bool {
        // TODO: Can we prevent the clone?
        let key = RequestKey {
            peer_id: peer_id.clone(),
            request_id: TypeId::of::<R>(),
        };

        let mut requests = self.open_requests.write().expect("error getting read access");
        (*requests).remove(&key).is_some()
    }
}

#[async_trait::async_trait]
impl Repeat for RequestManager {
    type Command = Box<dyn Fn(&Self::Context) + Send>;
    type Context = Self;

    async fn repeat(mut delay: DelayFactory, cmd: Self::Command, ctx: Self::Context, mut shutdown_rx: ShutdownRx) {
        while let Some(duration) = delay.next() {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                _ = time::sleep(duration) => {
                    cmd(&ctx);
                }
            }
        }
    }
}

pub(crate) fn is_expired(timestamp: Timestamp) -> bool {
    // TODO: use time::since instead
    timestamp
        .checked_add(REQUEST_EXPIRATION_SECS)
        .expect("timestamp checked add")
        < time::unix_now_secs()
}
