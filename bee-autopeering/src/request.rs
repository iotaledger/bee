// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use num::CheckedAdd;
use tokio::sync::oneshot;

use crate::{
    delay::DelayFactory,
    discovery::messages::{DiscoveryRequest, VerificationRequest, VerificationResponse},
    hash,
    identity::PeerId,
    local::Local,
    packet::{msg_hash, MessageType},
    peering_messages::PeeringRequest,
    peerstore::PeerStore,
    salt::Salt,
    server::ServerTx,
    task::{Repeat, ShutdownRx},
    time::{self, Timestamp},
};

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    iter,
    net::{IpAddr, SocketAddr},
    ops::DerefMut,
    sync::{Arc, RwLock},
    time::Duration,
};

type RequestHash = [u8; hash::SHA256_LEN];
pub(crate) type Callback = Box<dyn Fn() + Send + Sync + 'static>;
pub(crate) type ResponseSignal = oneshot::Sender<()>;

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
    pub(crate) request_hash: [u8; hash::SHA256_LEN],
    pub(crate) expiration_time: u64,
    pub(crate) callback: Option<Callback>,
    pub(crate) response_signal: Option<ResponseSignal>,
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

    pub(crate) fn new_verification_request(
        &self,
        peer_id: PeerId,
        target_addr: IpAddr,
        callback: Option<Callback>,
        response_signal: Option<ResponseSignal>,
    ) -> VerificationRequest {
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
            callback,
            response_signal,
        };

        let _ = self
            .open_requests
            .write()
            .expect("error getting write access")
            .insert(key, value);

        verif_req
    }

    pub(crate) fn new_discovery_request(
        &self,
        peer_id: PeerId,
        target_addr: IpAddr,
        callback: Option<Callback>,
        response_signal: Option<ResponseSignal>,
    ) -> DiscoveryRequest {
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
            callback,
            response_signal,
        };

        let _ = self
            .open_requests
            .write()
            .expect("error getting write access")
            .insert(key, value);

        disc_req
    }

    pub(crate) fn new_peering_request(
        &self,
        peer_id: PeerId,
        callback: Option<Callback>,
        response_signal: Option<ResponseSignal>,
    ) -> PeeringRequest {
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
            callback,
            response_signal,
        };

        let _ = self
            .open_requests
            .write()
            .expect("error getting write access")
            .insert(key, value);

        peer_req
    }

    pub(crate) fn pull<R: Request + 'static>(&self, peer_id: &PeerId) -> Option<RequestValue> {
        // TODO: Can we prevent the clone?
        let key = RequestKey {
            peer_id: peer_id.clone(),
            request_id: TypeId::of::<R>(),
        };

        let mut requests = self.open_requests.write().expect("error getting read access");

        (*requests).remove(&key)
    }
}

pub(crate) fn is_expired(timestamp: Timestamp) -> bool {
    time::since(timestamp).map_or(false, |ts| ts >= REQUEST_EXPIRATION_SECS)
}

pub(crate) fn remove_expired_requests_repeat() -> Repeat<RequestManager> {
    Box::new(|mngr: &RequestManager| {
        // Retain only those, which expire in the future.
        mngr.open_requests
            .write()
            .expect("error getting write access")
            .retain(|_, v| v.expiration_time > time::unix_now_secs());

        if !mngr.open_requests.read().expect("error getting read access").is_empty() {
            log::warn!(
                "Open requests: {}",
                mngr.open_requests.read().expect("error getting read access").len()
            );
        }
    })
}
