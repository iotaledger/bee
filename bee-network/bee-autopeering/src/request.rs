// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    discovery::messages::{DiscoveryRequest, VerificationRequest},
    hash,
    hash::message_hash,
    local::Local,
    packet::MessageType,
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
    sync::{Arc, RwLock},
    time::Duration,
};

type RequestHash = [u8; hash::SHA256_LEN];
pub(crate) type ResponseTx = oneshot::Sender<Vec<u8>>;

// If the request is not answered within that time it gets removed from the manager, and any response
// coming in later will be deemed invalid.
pub(crate) const REQUEST_EXPIRATION: Duration = Duration::from_secs(20);
pub(crate) const EXPIRED_REQUEST_REMOVAL_INTERVAL: Duration = Duration::from_secs(1);
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
    pub(crate) issue_time: u64,
    pub(crate) response_tx: Option<ResponseTx>,
}

#[derive(Clone)]
pub(crate) struct RequestManager {
    version: u32,
    network_id: u32,
    source_addr: SocketAddr,
    open_requests: Arc<RwLock<HashMap<RequestKey, RequestValue>>>,
}

impl RequestManager {
    /// Creates a new request manager.
    pub(crate) fn new(version: u32, network_id: u32, source_addr: SocketAddr) -> Self {
        Self {
            version,
            network_id,
            source_addr,
            open_requests: Arc::new(RwLock::new(HashMap::default())),
        }
    }

    /// Creates and registers an open verification request.
    pub(crate) fn create_verification_request(
        &self,
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

        let request_hash = message_hash(MessageType::VerificationRequest, &verif_req.to_protobuf());

        let value = RequestValue {
            request_hash,
            issue_time: timestamp,
            response_tx,
        };

        let _ = self.open_requests.write().expect("write").insert(key, value);

        verif_req
    }

    /// Creates and registers an open discovery request.
    pub(crate) fn create_discovery_request(
        &self,
        peer_id: PeerId,
        response_tx: Option<ResponseTx>,
    ) -> DiscoveryRequest {
        let key = RequestKey {
            peer_id,
            request_id: TypeId::of::<DiscoveryRequest>(),
        };

        let disc_req = DiscoveryRequest::new();
        let timestamp = disc_req.timestamp();

        let request_hash = message_hash(MessageType::DiscoveryRequest, &disc_req.to_protobuf());

        let value = RequestValue {
            request_hash,
            issue_time: timestamp,
            response_tx,
        };

        let _ = self.open_requests.write().expect("write").insert(key, value);

        disc_req
    }

    /// Creates and registers an open peering request.
    pub(crate) fn create_peering_request(
        &self,
        peer_id: PeerId,
        response_tx: Option<ResponseTx>,
        local: &Local,
    ) -> PeeringRequest {
        let key = RequestKey {
            peer_id,
            request_id: TypeId::of::<PeeringRequest>(),
        };

        let peer_req = PeeringRequest::new(local.public_salt());

        let timestamp = peer_req.timestamp();

        let request_hash = message_hash(MessageType::PeeringRequest, &peer_req.to_protobuf());

        let value = RequestValue {
            request_hash,
            issue_time: timestamp,
            response_tx,
        };

        let _ = self.open_requests.write().expect("write").insert(key, value);

        peer_req
    }

    /// Removes a request to a peer.
    pub(crate) fn remove_request<R: Request + 'static>(&self, peer_id: &PeerId) -> Option<RequestValue> {
        let key = RequestKey {
            peer_id: *peer_id,
            request_id: TypeId::of::<R>(),
        };

        self.open_requests.write().expect("write").remove(&key)
    }

    /// Removes all expired requests.
    pub(crate) fn remove_expired_requests(&self, now_ts: u64) {
        self.open_requests
            .write()
            .expect("write")
            .retain(|_, v| !is_expired_internal(v.issue_time, now_ts));
    }

    /// Returns the number of open requests.
    pub(crate) fn num_open_requests(&self) -> usize {
        self.open_requests.read().expect("read").len()
    }
}

pub(crate) fn is_expired(past_ts: Timestamp) -> bool {
    is_expired_internal(past_ts, time::unix_now_secs())
}

fn is_expired_internal(past_ts: Timestamp, now_ts: Timestamp) -> bool {
    // Note: `time::since` returns `None` for a timestamp that lies in the future, hence it cannot be expired yet,
    // and must therefore be mapped to `false` (not expired).
    time::delta(past_ts, now_ts).map_or(false, |span| span >= REQUEST_EXPIRATION.as_secs())
}

pub(crate) fn remove_expired_requests_fn() -> Repeat<RequestManager> {
    Box::new(|mngr: &RequestManager| {
        let now_ts = time::unix_now_secs();

        // Retain only those that aren't expired yet, remove all others.
        mngr.remove_expired_requests(now_ts);

        let num_open_requests = mngr.num_open_requests();
        if num_open_requests > 0 {
            log::trace!("Open requests: {}", num_open_requests);
        }
    })
}
