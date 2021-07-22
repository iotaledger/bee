// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::PluginError,
    event::*,
    grpc::{
        plugin_server::{Plugin as GrpcPlugin, PluginServer},
        HandshakeReply, HandshakeRequest, ProcessReply, ShutdownReply, ShutdownRequest,
    },
};

use tonic::{transport::Server, Request, Response, Status, Streaming};

macro_rules! plugin_trait {
    ($($method_name:ident => $event_ty:ty),*) => {
        #[tonic::async_trait]
        pub trait Plugin: Send + Sync + 'static {
            fn handshake() -> Vec<EventId>;
            async fn shutdown(&self);
            $(async fn $method_name(&self, _event: $event_ty) {})*
        }

        #[tonic::async_trait]
        impl<T: Plugin> GrpcPlugin for T {
            async fn handshake(&self, _request: Request<HandshakeRequest>) -> Result<Response<HandshakeReply>, Status> {
                Ok(Response::new(HandshakeReply {
                    ids: Self::handshake()
                        .into_iter()
                        .map(|event_id| event_id.into())
                        .collect(),
                }))
            }

            async fn shutdown(&self, _request: Request<ShutdownRequest>) -> Result<Response<ShutdownReply>, Status> {
                self.shutdown().await;
                Ok(Response::new(ShutdownReply {}))
            }

            $(
                async fn $method_name(
                    &self,
                    request: Request<Streaming<$event_ty>>,
                ) -> Result<Response<ProcessReply>, Status> {
                    let mut stream = request.into_inner();

                    while let Some(message) = stream.message().await? {
                        self.$method_name(message).await;
                    }

                    Ok(Response::new(ProcessReply {}))
                }
            )*
        }
    };
}

plugin_trait! {
    process_dummy_event => DummyEvent,
    process_silly_event => SillyEvent
}

pub async fn serve_plugin<T: Plugin>(plugin: T) -> Result<(), PluginError> {
    let addr = "[::1]:50051".parse()?;

    Server::builder()
        .add_service(PluginServer::new(plugin))
        .serve(addr)
        .await?;

    Ok(())
}
