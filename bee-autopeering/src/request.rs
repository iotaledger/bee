// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    delay::{Delay, Repeat},
    discovery_messages::{DiscoveryRequest, VerificationRequest},
    identity::PeerId,
    peering_messages::PeeringRequest,
    salt::Salt,
    time,
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

// If the request is not answered within that time it gets removed from the manager.
const REQUEST_EXPIRATION_SECS: u64 = 20;

// FIXME
const SALT_DURATION: Duration = Duration::from_secs(3600);

// Marker trait for requests.
pub(crate) trait Request: Debug + Clone {}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) struct RequestKey {
    pub(crate) peer_id: PeerId,
    pub(crate) request_id: TypeId,
}

pub(crate) struct RequestValue {
    pub(crate) request: Box<dyn Any + Send + Sync>,
    pub(crate) expiration_time: u64,
}

#[derive(Clone)]
pub(crate) struct RequestManager {
    pub(crate) version: u32,
    pub(crate) network_id: u32,
    pub(crate) source_addr: SocketAddr,
    pub(crate) salt: Arc<RwLock<Salt>>,
    pub(crate) open_requests: Arc<RwLock<HashMap<RequestKey, RequestValue>>>,
}

impl RequestManager {
    pub(crate) fn new(version: u32, network_id: u32, source_addr: SocketAddr) -> Self {
        Self {
            version,
            network_id,
            source_addr,
            salt: Arc::new(RwLock::new(Salt::new(SALT_DURATION))),
            open_requests: Arc::new(RwLock::new(HashMap::default())),
        }
    }

    pub(crate) fn new_verification_request(&self, target: IpAddr) -> VerificationRequest {
        let timestamp = crate::time::unix_now();

        VerificationRequest {
            version: self.version,
            network_id: self.network_id,
            timestamp,
            source_addr: self.source_addr,
            target_addr: target,
        }
    }

    pub(crate) fn new_discovery_request(&self) -> DiscoveryRequest {
        let timestamp = crate::time::unix_now();

        DiscoveryRequest { timestamp }
    }

    pub(crate) fn new_peering_request(&self) -> PeeringRequest {
        let timestamp = crate::time::unix_now();

        PeeringRequest {
            timestamp,
            salt: self.salt.read().expect("error getting read access").clone(),
        }
    }

    pub(crate) fn insert_request(&self, peer_id: PeerId, request: Box<dyn Any + Send + Sync>) {
        let mut guard = self.open_requests.write().expect("error getting write access");
        let requests = guard.deref_mut();
        let request_id = request.type_id();
        let request_key = RequestKey { peer_id, request_id };
        let request_value = RequestValue {
            request,
            expiration_time: time::unix_now() + REQUEST_EXPIRATION_SECS,
        };

        requests.insert(request_key, request_value);
    }

    pub(crate) fn get_request<R: Request + 'static>(&self, peer_id: PeerId) -> Option<R> {
        let key = RequestKey {
            peer_id,
            request_id: TypeId::of::<R>(),
        };

        let requests = self.open_requests.read().expect("error getting read access");
        if let Some(RequestValue { request, .. }) = (*requests).get(&key) {
            if let Some(request) = request.downcast_ref::<R>() {
                Some(request.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[async_trait::async_trait]
impl Repeat for RequestManager {
    type Command = Box<dyn Fn(&Self::Context) + Send>;
    type Context = Self;

    async fn repeat(mut delay: Delay, cmd: Self::Command, ctx: Self::Context) {
        while let Some(duration) = delay.next() {
            time::sleep(duration).await;
            cmd(&ctx);
        }
    }
}
