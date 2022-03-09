// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use fern_logger::LoggerConfig;
use serde::Deserialize;
use trace_tools::{subscriber, util::Flamegrapher};

/// Default folded stack filename.
const DEFAULT_STACK_FILENAME: &str = "stack.folded";
/// Default flamegraph filename.
const DEFAULT_GRAPH_FILENAME: &str = "flamegraph.svg";

/// Builder for the tracing configuration.
#[derive(Default, Deserialize, PartialEq)]
#[must_use]
pub struct TraceConfigBuilder {
    /// Enables the console layer.
    #[serde(alias = "consoleEnabled")]
    pub console_enabled: bool,

    /// Options for the flamegraph layer. If this is present, the layer is enabled.
    pub flamegraph: Option<FlamegraphConfigBuilder>,
}

impl TraceConfigBuilder {
    /// Builds the tracing configuration.
    pub fn finish(self) -> TraceConfig {
        TraceConfig {
            console_enabled: self.console_enabled,
            flamegraph: self.flamegraph.map(|builder| builder.finish()),
        }
    }
}

/// Tracing configuration options.
#[derive(Clone)]
pub struct TraceConfig {
    /// Enables the console layer.
    pub console_enabled: bool,

    /// Options for the flamegraph layer.
    pub flamegraph: Option<FlamegraphConfig>,
}

/// Builder for the flamegraph layer configuration.
#[derive(Default, Deserialize, PartialEq)]
#[must_use]
pub struct FlamegraphConfigBuilder {
    /// Folded stack filename, relative to the crate root. If not present, defaults to "stack.folded".
    #[serde(alias = "stackFilename")]
    pub stack_filename: Option<String>,

    /// Flamegraph filename, relative to the crate root. If not present, defaults to "flamegraph.svg".
    #[serde(alias = "graphFilename")]
    pub graph_filename: Option<String>,
}

impl FlamegraphConfigBuilder {
    /// Builds the flamegraph layer configuration.
    pub fn finish(self) -> FlamegraphConfig {
        FlamegraphConfig {
            stack_filename: self
                .stack_filename
                .unwrap_or_else(|| DEFAULT_STACK_FILENAME.to_string()),
            graph_filename: self
                .graph_filename
                .unwrap_or_else(|| DEFAULT_GRAPH_FILENAME.to_string()),
        }
    }
}

/// Flamegraph layer configuration options.
#[derive(Clone)]
pub struct FlamegraphConfig {
    /// Folded stack filename, relative to the crate root.
    pub stack_filename: String,

    /// Flamegraph filename, relative to the crate root.
    pub graph_filename: String,
}

/// Initialise tracing features with a given configuration.
pub fn init(
    logger_config: LoggerConfig,
    tracing_config: TraceConfig,
) -> Result<Option<Flamegrapher>, trace_tools::Error> {
    let mut builder = subscriber::build().with_log_layer(logger_config);

    if tracing_config.console_enabled {
        builder = builder.with_console_layer();
    }

    if let Some(flamegraph_config) = tracing_config.flamegraph {
        builder = builder.with_flamegraph_layer(flamegraph_config.stack_filename);

        let flamegrapher = builder.init()?;

        if let Some(flamegrapher) = flamegrapher {
            return Ok(Some(flamegrapher.with_graph_file(flamegraph_config.graph_filename)?));
        } else {
            return Ok(flamegrapher);
        }
    }

    builder.init()
}
