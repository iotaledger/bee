mod flamegraph;
mod log;

pub use self::{flamegraph::FlamegraphLayer, log::LogLayer};

use crate::{util::Flamegrapher, Error};

use bee_common::logger::LoggerConfig;

use tracing::Metadata;
use tracing_subscriber::{
    filter::{FilterFn, Filtered},
    registry::Registry,
    Layer,
};

use std::path::Path;

pub type FlamegraphFilteredLayer = Filtered<FlamegraphLayer, FilterFn, Registry>;

pub fn flamegraph_layer<P: AsRef<Path>>(stack_filename: P) -> Result<(FlamegraphFilteredLayer, Flamegrapher), Error> {
    #![allow(clippy::assertions_on_constants)]
    assert!(
        cfg!(tokio_unstable),
        "task tracing requires building with RUSTFLAGS=\"--cfg tokio_unstable\"!"
    );

    fn filter_fn(meta: &Metadata<'_>) -> bool {
        if meta.is_event() {
            return false;
        }

        meta.target().starts_with("runtime") || meta.target().starts_with("tokio") || meta.target() == "bee::observe"
    }

    let filter = FilterFn::new(filter_fn as for<'r, 's> fn(&'r tracing::Metadata<'s>) -> bool);
    let (layer, flamegrapher) = FlamegraphLayer::new(stack_filename)
        .map(|(layer, _flamegrapher)| (layer.with_filter(filter), _flamegrapher))?;

    Ok((layer, flamegrapher))
}

pub fn log_layer(config: LoggerConfig) -> Result<LogLayer, Error> {
    LogLayer::new(config)
}
