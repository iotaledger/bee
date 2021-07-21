// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::grpc::{
    plugin_server::Plugin as GrpcPlugin, HandshakeReply, HandshakeRequest, ProcessReply, ShutdownReply, ShutdownRequest,
};
pub use crate::grpc::{DummyEvent, EventId};

use tonic::{Request, Response, Status, Streaming};

#[tonic::async_trait]
pub trait Plugin: Send + Sync + 'static {
    fn handshake() -> Vec<EventId>;
    async fn shutdown(&self);
    async fn process_dummy_event(&self, event: DummyEvent);
}

#[tonic::async_trait]
impl<T: Plugin> GrpcPlugin for T {
    async fn handshake(&self, _request: Request<HandshakeRequest>) -> Result<Response<HandshakeReply>, Status> {
        Ok(Response::new(HandshakeReply {
            ids: Self::handshake().into_iter().map(|event_id| event_id.into()).collect(),
        }))
    }

    async fn shutdown(&self, _request: Request<ShutdownRequest>) -> Result<Response<ShutdownReply>, Status> {
        self.shutdown().await;
        Ok(Response::new(ShutdownReply {}))
    }

    async fn process_dummy_event(
        &self,
        request: Request<Streaming<DummyEvent>>,
    ) -> Result<Response<ProcessReply>, Status> {
        let mut stream = request.into_inner();
        while let Some(event) = stream.message().await? {
            self.process_dummy_event(event).await;
        }

        Ok(Response::new(ProcessReply {}))
    }
}
