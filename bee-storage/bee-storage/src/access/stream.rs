// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::storage::Backend;

use futures::stream::Stream;

/// AsStream<'a, K, V> trait will extend the Backend with Scan operation for the key: K value: V collection
/// therefore it should be explicitly implemented for the corresponding Backend.
#[async_trait::async_trait]
pub trait AsStream<'a, K, V>: Backend {
    type Stream: Stream;
    /// This method returns the Stream object for the provided <K, V> collection in order to later execute async next()
    /// calls
    async fn stream(&'a self) -> Result<Self::Stream, Self::Error>
    where
        Self: Sized;
}
