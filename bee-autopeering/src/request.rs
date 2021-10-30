// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    delay::DelayFactory,
    discovery::{
        manager::DiscoveryHandler,
        messages::{DiscoveryRequest, DiscoveryResponse, VerificationRequest, VerificationResponse},
    },
    hash,
    local::{salt::Salt, Local},
    packet::{msg_hash, MessageType},
    peer::{peer_id::PeerId, peerstore::PeerStore},
    peering::{manager::PeeringHandler, messages::PeeringRequest},
    server::ServerTx,
    task::{Repeat, ShutdownRx},
    time::{self, Timestamp},
};

use num::CheckedAdd;
use tokio::sync::oneshot;

pub(crate) use oneshot::channel as response_chan;

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

pub(crate) struct RequestValue<S: PeerStore> {
    pub(crate) request_hash: [u8; hash::SHA256_LEN],
    pub(crate) expiration_time: u64,
    pub(crate) handler: Option<DiscoveryHandler<S>>,
    pub(crate) response_tx: Option<ResponseTx>,
}

#[derive(Clone)]
pub(crate) struct RequestManager<S: PeerStore> {
    pub(crate) version: u32,
    pub(crate) network_id: u32,
    pub(crate) source_addr: SocketAddr,
    pub(crate) local: Local,
    pub(crate) open_requests: Arc<RwLock<HashMap<RequestKey, RequestValue<S>>>>,
}

impl<S: PeerStore> RequestManager<S> {
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
        peer_addr: IpAddr,
        handler: Option<DiscoveryHandler<S>>,
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
            handler,
            response_tx,
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
        handler: Option<DiscoveryHandler<S>>,
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
            handler,
            response_tx,
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
        handler: Option<PeeringHandler<S>>,
        response_tx: Option<ResponseTx>,
    ) -> PeeringRequest {
        let key = RequestKey {
            peer_id,
            request_id: TypeId::of::<PeeringRequest>(),
        };

        let peer_req = PeeringRequest::new(self.local.read().public_salt().expect("missing public salt").clone());

        let timestamp = peer_req.timestamp();

        let request_hash = msg_hash(
            MessageType::PeeringRequest,
            &peer_req.to_protobuf().expect("error encoding peering request"),
        );

        let value = RequestValue {
            request_hash,
            expiration_time: timestamp + REQUEST_EXPIRATION_SECS,
            // TODO: support PeeringHandler
            handler: None,
            response_tx,
        };

        let _ = self
            .open_requests
            .write()
            .expect("error getting write access")
            .insert(key, value);

        peer_req
    }

    pub(crate) fn pull<R: Request + 'static>(&self, peer_id: &PeerId) -> Option<RequestValue<S>> {
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

pub(crate) fn remove_expired_requests_repeat<S: PeerStore>() -> Repeat<RequestManager<S>> {
    Box::new(|mngr: &RequestManager<S>| {
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
