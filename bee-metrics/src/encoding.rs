// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types and traits related to metrics encoding.

use std::ops::Deref;

pub use prometheus_client::{
    encoding::text::{EncodeMetric, Encoder, SendEncodeMetric},
    metrics::MetricType,
};

/// Trait implemented by each metric type that also is [`Send`] and [`Sync`].
pub trait SendSyncEncodeMetric: SendEncodeMetric + Sync {}

impl<T: SendEncodeMetric + Sync> SendSyncEncodeMetric for T {}

impl EncodeMetric for Box<dyn SendSyncEncodeMetric> {
    #[inline(always)]
    fn encode(&self, encoder: Encoder) -> Result<(), std::io::Error> {
        self.deref().encode(encoder)
    }

    #[inline(always)]
    fn metric_type(&self) -> MetricType {
        self.deref().metric_type()
    }
}
