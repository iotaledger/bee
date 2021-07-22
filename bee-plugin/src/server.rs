// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub use crate::grpc::{DummyEvent, SillyEvent};
use crate::{
    error::PluginError,
    grpc::{
        plugin_server::{Plugin as GrpcPlugin, PluginServer},
        EventId, HandshakeReply, HandshakeRequest, ProcessReply, ShutdownReply, ShutdownRequest,
    },
};

use tonic::{transport::Server, Request, Response, Status, Streaming};

#[tonic::async_trait]
pub trait Plugin: Send + Sync + 'static {
    fn handshake() -> Vec<EventId>;
    async fn shutdown(&self);
    async fn process_dummy_event(&self, _event: DummyEvent) {}
    async fn process_silly_event(&self, _event: SillyEvent) {}
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

        while let Some(message) = stream.message().await? {
            self.process_dummy_event(message).await;
        }

        Ok(Response::new(ProcessReply {}))
    }

    async fn process_silly_event(
        &self,
        request: Request<Streaming<SillyEvent>>,
    ) -> Result<Response<ProcessReply>, Status> {
        let mut stream = request.into_inner();

        while let Some(message) = stream.message().await? {
            self.process_silly_event(message).await;
        }

        Ok(Response::new(ProcessReply {}))
    }
}

pub async fn serve_plugin<T: Plugin>(plugin: T) -> Result<(), PluginError> {
    let addr = "[::1]:50051".parse()?;

    Server::builder()
        .add_service(PluginServer::new(plugin))
        .serve(addr)
        .await?;

    Ok(())
}
