use std::{ops::Deref, sync::Arc};

use parking_lot::RwLock;
use prometheus_client::{
    encoding::text::{EncodeMetric, Encoder, SendEncodeMetric},
    metrics::MetricType,
    registry::Registry as PrometheusRegistry,
};

pub trait SendSyncEncodeMetric: SendEncodeMetric + Sync {}

impl<T: SendEncodeMetric + Sync> SendSyncEncodeMetric for T {}

impl EncodeMetric for Box<dyn SendSyncEncodeMetric> {
    fn encode(&self, encoder: Encoder) -> Result<(), std::io::Error> {
        self.deref().encode(encoder)
    }

    fn metric_type(&self) -> MetricType {
        self.deref().metric_type()
    }
}

#[derive(Clone)]
pub struct Registry {
    pub(crate) registry: Arc<RwLock<PrometheusRegistry<Box<dyn SendSyncEncodeMetric + 'static>>>>,
}

impl Registry {
    pub fn register(
        &self,
        name: impl Into<String>,
        help: impl Into<String>,
        metric: impl SendSyncEncodeMetric + 'static,
    ) {
        self.registry.write().register(name, help, Box::new(metric))
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            registry: Arc::new(RwLock::new(Default::default())),
        }
    }
}
