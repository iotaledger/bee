// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::identity::PeerId;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

// If the request is not answered within that time it gets removed from the manager.
const REQUEST_EXPIRATION_SECS: u64 = 20;

pub(crate) trait Request {
    type Data;
    type Response;
    type ResponseHandler;

    fn handle_response(&self, _: Self::Data, _: Self::Response, _: Self::ResponseHandler);
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
struct RequestKey {
    peer_id: PeerId,
    request_id: TypeId,
}

struct RequestValue {
    request: Box<dyn Any + Send>,
    expiration_time: SystemTime,
}

pub(crate) struct RequestManager {
    requests: HashMap<RequestKey, RequestValue>,
}

impl RequestManager {
    pub(crate) fn new() -> Self {
        Self {
            requests: HashMap::default(),
        }
    }

    pub(crate) fn get_request<R: Request + 'static>(&self, peer_id: PeerId) -> Option<&R> {
        let key = RequestKey {
            peer_id,
            request_id: TypeId::of::<R>(),
        };

        if let Some(RequestValue { request, .. }) = self.requests.get(&key) {
            if let Some(request) = request.downcast_ref::<R>() {
                Some(request)
            } else {
                None
            }
        } else {
            None
        }
    }
}
