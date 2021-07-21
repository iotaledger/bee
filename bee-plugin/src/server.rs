// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

pub use crate::grpc::DummyEvent;
use crate::grpc::{
    plugin_server::{Plugin as GrpcPlugin, PluginServer},
    EventId, HandshakeReply, HandshakeRequest, ProcessReply, ShutdownReply, ShutdownRequest,
};

use tonic::{transport::Server, Request, Response, Status};

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

    async fn process_dummy_event(&self, request: Request<DummyEvent>) -> Result<Response<ProcessReply>, Status> {
        self.process_dummy_event(request.into_inner()).await;

        Ok(Response::new(ProcessReply {}))
    }
}

pub async fn serve_plugin<T: Plugin>(plugin: T) -> Result<(), Box<dyn Error>> {
    let addr = "[::1]:50051".parse()?;

    println!("Server is running");

    Server::builder()
        .add_service(PluginServer::new(plugin))
        .serve(addr)
        .await?;

    Ok(())
}
