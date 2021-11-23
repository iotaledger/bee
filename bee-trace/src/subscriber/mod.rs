pub mod layer;

use crate::{util::Flamegrapher, Error};

use bee_common::logger::LoggerConfig;

use tracing_log::LogTracer;
use tracing_subscriber::{layer::Layered, prelude::*, Registry};

use std::path::{Path, PathBuf};

pub fn collect_logs() -> Result<(), log::SetLoggerError> {
    LogTracer::init()
}

pub type BeeSubscriber = Layered<Option<layer::LogLayer>, Layered<Option<layer::FlamegraphFilteredLayer>, Registry>>;

#[derive(Default)]
pub struct SubscriberBuilder {
    logger_config: Option<LoggerConfig>,
    flamegraph_stack_file: Option<PathBuf>,
}

impl SubscriberBuilder {
    pub fn with_log_layer(mut self, logger_config: LoggerConfig) -> Self {
        self.logger_config = Some(logger_config);
        self
    }

    pub fn with_flamegraph_layer<P: AsRef<Path>>(mut self, folded_stack_file: P) -> Self {
        self.flamegraph_stack_file = Some(folded_stack_file.as_ref().to_path_buf());
        self
    }

    pub fn finish(mut self) -> Result<(BeeSubscriber, Option<Flamegrapher>), Error> {
        let (flamegraph_layer, flamegrapher) = self.build_flamegraph_layer()?;
        let log_layer = self.build_log_layer()?;

        let subscriber = tracing_subscriber::registry().with(flamegraph_layer).with(log_layer);

        Ok((subscriber, flamegrapher))
    }

    pub fn init(mut self) -> Result<Option<Flamegrapher>, Error> {
        let (flamegraph_layer, flamegrapher) = self.build_flamegraph_layer()?;
        let log_layer = self.build_log_layer()?;

        let subscriber = tracing_subscriber::registry().with(flamegraph_layer).with(log_layer);

        subscriber.init();

        Ok(flamegrapher)
    }

    fn build_log_layer(&mut self) -> Result<Option<layer::LogLayer>, Error> {
        if self.logger_config.is_some() {
            collect_logs().map_err(|err| Error::LogLayer(err.into()))?;
        }

        self.logger_config
            .take()
            .map(|config| layer::log_layer(config))
            .map_or(Ok(None), |res| res.map(Some))
    }

    fn build_flamegraph_layer(
        &mut self,
    ) -> Result<(Option<layer::FlamegraphFilteredLayer>, Option<Flamegrapher>), Error> {
        self.flamegraph_stack_file
            .take()
            .map(|stack_filename| layer::flamegraph_layer(stack_filename))
            .map_or(Ok((None, None)), |res| {
                res.map(|(layer, flamegrapher)| (Some(layer), Some(flamegrapher)))
            })
    }
}

pub fn build() -> SubscriberBuilder {
    SubscriberBuilder::default()
}
