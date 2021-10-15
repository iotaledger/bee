// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{cron::CronJob, identity::PeerId, time};

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    pin::Pin,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

// If the request is not answered within that time it gets removed from the manager.
const REQUEST_EXPIRATION_SECS: u64 = 20;

pub(crate) trait Request {
    type Data;
    type Response;
    type ResponseHandler;

    fn handle_response(&self, _: Self::Data, _: Self::Response, _: Self::ResponseHandler);
}

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
    pub(crate) requests: Arc<RwLock<HashMap<RequestKey, RequestValue>>>,
}

impl RequestManager {
    pub(crate) fn new() -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::default())),
        }
    }

    pub(crate) fn get_request<R: Request + Clone + 'static>(&self, peer_id: PeerId) -> Option<R> {
        let key = RequestKey {
            peer_id,
            request_id: TypeId::of::<R>(),
        };

        let requests = self.requests.read().expect("error getting read lock");
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
impl CronJob for RequestManager {
    type Command = Box<dyn Fn(&Self) + Send>;
    type Data = ();

    async fn cronjob(self, period: Duration, cmd: Self::Command, _: Self::Data) {
        let mut interval = time::interval(period);
        loop {
            interval.tick().await;
            cmd(&self);
        }
    }
}
