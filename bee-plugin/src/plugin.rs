// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Utilities to write plugins.

pub use crate::handshake::HandshakeInfo;
use crate::{
    error::PluginError,
    event::*,
    grpc::{
        plugin_server::{Plugin as GrpcPlugin, PluginServer},
        ProcessReply, ShutdownReply, ShutdownRequest,
    },
};

use tokio::io::{stdout, AsyncWriteExt};
use tonic::{transport::Server, Request, Response, Status, Streaming};

macro_rules! plugin_trait {
    ($($method_name:ident => $event_ty:ty),*) => {
        /// Types that represent a plugin.
        #[tonic::async_trait]
        pub trait Plugin: Send + Sync + 'static {
            /// Returns the `HandshakeInfo` of the current plugin.
            fn handshake_info() -> HandshakeInfo;
            /// Prepares the plugin for shutdown.
            async fn shutdown(&self);
            $(
                #[doc = "Handles events the specified type."]
                async fn $method_name(&self, _event: $event_ty) {}
            )*
        }

        #[tonic::async_trait]
        impl<T: Plugin> GrpcPlugin for T {
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

/// Does the handshake and runs a gRPC server for the specified plugin.
pub async fn serve_plugin<T: Plugin>(plugin: T) -> Result<(), PluginError> {
    let handshake_info = T::handshake_info();
    let address = handshake_info.address;

    stdout().write_all(handshake_info.emit().as_bytes()).await?;

    Server::builder()
        .add_service(PluginServer::new(plugin))
        .serve(address)
        .await?;

    Ok(())
}
