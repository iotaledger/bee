// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that deals with asynchronous workers in general.

use crate::node::Node;

use async_trait::async_trait;

use std::any::{Any, TypeId};

/// Errors that might occur during the lifetime of asynchronous workers.
#[derive(Debug)]
pub struct Error(pub Box<dyn std::error::Error + Send>);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Worker error: {:?}.", self.0)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

#[async_trait]
pub trait Worker<N: Node>: Any + Send + Sync + Sized {
    type Config;
    type Error: std::error::Error;

    // TODO Replace with associated constant when stabilized.
    fn dependencies() -> &'static [TypeId] {
        &[]
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error>;

    async fn stop(self, _node: &mut N) -> Result<(), Self::Error> {
        Ok(())
    }
}
