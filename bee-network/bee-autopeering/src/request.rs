// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    discovery::messages::{DiscoveryRequest, VerificationRequest},
    hash,
    local::Local,
    packet::{msg_hash, MessageType},
    peer::peer_id::PeerId,
    peering::messages::PeeringRequest,
    task::Repeat,
    time::{self, Timestamp},
};

use tokio::sync::oneshot;

pub(crate) use oneshot::channel as response_chan;

use std::{
    any::TypeId,
    collections::HashMap,
    fmt::Debug,
    net::{IpAddr, SocketAddr},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};

type RequestHash = [u8; hash::SHA256_LEN];
pub(crate) type ResponseTx = oneshot::Sender<Vec<u8>>;

// If the request is not answered within that time it gets removed from the manager, and any response
// coming in later will be deemed invalid.
pub(crate) const REQUEST_EXPIRATION_SECS: u64 = 20;
pub(crate) const EXPIRED_REQUEST_REMOVAL_INTERVAL_CHECK_SECS: u64 = 1;
pub(crate) const RESPONSE_TIMEOUT: Duration = Duration::from_millis(500);

// Marker trait for requests.
pub(crate) trait Request: Debug + Clone {}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) struct RequestKey {
    pub(crate) peer_id: PeerId,
    pub(crate) request_id: TypeId,
}

pub(crate) struct RequestValue {
    pub(crate) request_hash: RequestHash,
    pub(crate) expiration_time: u64,
    pub(crate) response_tx: Option<ResponseTx>,
}

#[derive(Clone)]
pub(crate) struct RequestManager {
    inner: Arc<RwLock<RequestManagerInner>>,
}

impl RequestManager {
    /// Creates a new request manager.
    pub(crate) fn new(version: u32, network_id: u32, source_addr: SocketAddr) -> Self {
        Self {
            inner: Arc::new(RwLock::new(RequestManagerInner {
                version,
                network_id,
                source_addr,
                open_requests: HashMap::default(),
            })),
        }
    }
    pub(crate) fn read(&self) -> RwLockReadGuard<RequestManagerInner> {
        self.inner.read().expect("error getting read access")
    }

    pub(crate) fn write(&self) -> RwLockWriteGuard<RequestManagerInner> {
        self.inner.write().expect("error getting write access")
    }
}

pub(crate) struct RequestManagerInner {
    version: u32,
    network_id: u32,
    source_addr: SocketAddr,
    open_requests: HashMap<RequestKey, RequestValue>,
}

impl RequestManagerInner {
    pub(crate) fn new_verification_request(
        &mut self,
        peer_id: PeerId,
        peer_addr: IpAddr,
        response_tx: Option<ResponseTx>,
    ) -> VerificationRequest {
        let key = RequestKey {
            peer_id,
            request_id: TypeId::of::<VerificationRequest>(),
        };

        let verif_req = VerificationRequest::new(self.version, self.network_id, self.source_addr, peer_addr);
        let timestamp = verif_req.timestamp();

        let request_hash = msg_hash(
            MessageType::VerificationRequest,
            &verif_req.to_protobuf().expect("error encoding verification request"),
        );

        let value = RequestValue {
            request_hash,
            expiration_time: timestamp + REQUEST_EXPIRATION_SECS,
            response_tx,
        };

        let _ = self.open_requests.insert(key, value);

        verif_req
    }

    pub(crate) fn new_discovery_request(
        &mut self,
        peer_id: PeerId,
        response_tx: Option<ResponseTx>,
    ) -> DiscoveryRequest {
        let key = RequestKey {
            peer_id,
            request_id: TypeId::of::<DiscoveryRequest>(),
        };

        let disc_req = DiscoveryRequest::new();
        let timestamp = disc_req.timestamp();

        let request_hash = msg_hash(
            MessageType::DiscoveryRequest,
            &disc_req.to_protobuf().expect("error encoding discovery request"),
        );

        let value = RequestValue {
            request_hash,
            expiration_time: timestamp + REQUEST_EXPIRATION_SECS,
            response_tx,
        };

        let _ = self.open_requests.insert(key, value);

        disc_req
    }

    pub(crate) fn new_peering_request(
        &mut self,
        peer_id: PeerId,
        response_tx: Option<ResponseTx>,
        local: &Local,
    ) -> PeeringRequest {
        let key = RequestKey {
            peer_id,
            request_id: TypeId::of::<PeeringRequest>(),
        };

        let peer_req = PeeringRequest::new(local.public_salt().expect("missing public salt"));

        let timestamp = peer_req.timestamp();

        let request_hash = msg_hash(
            MessageType::PeeringRequest,
            &peer_req.to_protobuf().expect("error encoding peering request"),
        );

        let value = RequestValue {
            request_hash,
            expiration_time: timestamp + REQUEST_EXPIRATION_SECS,
            response_tx,
        };

        let _ = self.open_requests.insert(key, value);

        peer_req
    }

    pub(crate) fn pull<R: Request + 'static>(&mut self, peer_id: &PeerId) -> Option<RequestValue> {
        let key = RequestKey {
            peer_id: *peer_id,
            request_id: TypeId::of::<R>(),
        };

        self.open_requests.remove(&key)
    }
}

pub(crate) fn is_expired(timestamp: Timestamp) -> bool {
    time::since(timestamp).map_or(false, |ts| ts >= REQUEST_EXPIRATION_SECS)
}

pub(crate) fn remove_expired_requests_repeat() -> Repeat<RequestManager> {
    Box::new(|mngr: &RequestManager| {
        // Retain only those that aren't expired yet, remove all others.
        mngr.write()
            .open_requests
            .retain(|_, v| v.expiration_time > time::unix_now_secs());

        let num_open_requests = mngr.read().open_requests.len();
        if num_open_requests > 0 {
            log::trace!("Open requests: {}", num_open_requests);
        }
    })
}
