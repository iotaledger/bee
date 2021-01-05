// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::backend::StorageBackend;

use futures::stream::Stream;

/// `AsStream<'a, K, V>` trait extends the `StorageBackend` with `stream` operation for the (key: K, value: V) pair;
/// therefore, it should be explicitly implemented for the corresponding `StorageBackend`.
#[async_trait::async_trait]
pub trait AsStream<'a, K, V>: StorageBackend {
    /// Type to iterate through the <K, V> collection.
    type Stream: Stream;

    /// Returns a `Stream` object for the provided <K, V> collection.
    async fn stream(&'a self) -> Result<Self::Stream, Self::Error>;
}
