// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{fs, path::PathBuf};

use fern_logger::LoggerConfig;
use serde::Deserialize;
use trace_tools::{subscriber, util::Flamegrapher};

/// Default flamegraph output directory.
const DEFAULT_FLAMEGRAPH_OUT_PATH: &str = "./flamegraph";
/// Default folded stack filename.
const DEFAULT_STACK_FILENAME: &str = "stack.folded";
/// Default flamegraph filename.
const DEFAULT_GRAPH_FILENAME: &str = "flamegraph";

/// Builder for the tracing configuration.
#[derive(Default, Deserialize, PartialEq)]
#[must_use]
pub struct TraceConfigBuilder {
    /// Enables the console layer.
    #[serde(alias = "consoleEnabled", default)]
    pub console_enabled: bool,
    /// Enables the flamegraph layer.
    #[serde(alias = "flamegraphEnabled", default)]
    pub flamegraph_enabled: bool,
    /// Specifies the output directory of the flamegraph layer.
    #[serde(alias = "flamegraphOutputPath")]
    pub flamegraph_output_path: Option<String>,
}

impl TraceConfigBuilder {
    /// Builds the tracing configuration.
    pub fn finish(self) -> TraceConfig {
        TraceConfig {
            console_enabled: self.console_enabled,
            flamegraph_enabled: self.flamegraph_enabled,
            flamegraph_output_path: PathBuf::from(
                self.flamegraph_output_path
                    .unwrap_or_else(|| DEFAULT_FLAMEGRAPH_OUT_PATH.to_string()),
            ),
        }
    }
}

/// Tracing configuration options.
#[derive(Clone)]
pub struct TraceConfig {
    /// Enables the console layer.
    pub console_enabled: bool,
    /// Enables the flamegraph layer.
    pub flamegraph_enabled: bool,
    /// Specifies the output directory of the flamegraph layer.
    pub flamegraph_output_path: PathBuf,
}

/// Initialise tracing features with a given configuration.
pub fn init(
    logger_config: LoggerConfig,
    tracing_config: TraceConfig,
) -> Result<Option<Flamegrapher>, trace_tools::Error> {
    #![allow(clippy::assertions_on_constants)]
    assert!(
        cfg!(tokio_unstable),
        "`trace` feature requires building with RUSTFLAGS=\"--cfg tokio_unstable\"!"
    );

    let mut builder = subscriber::build().with_log_layer(logger_config);

    if tracing_config.console_enabled {
        builder = builder.with_console_layer();
    }

    if tracing_config.flamegraph_enabled {
        fs::create_dir_all(tracing_config.flamegraph_output_path.clone())
            .map_err(|err| trace_tools::Error::Flamegrapher(err.into()))?;

        let stack_filename = tracing_config.flamegraph_output_path.join(DEFAULT_STACK_FILENAME);
        let graph_filename = tracing_config.flamegraph_output_path.join(DEFAULT_GRAPH_FILENAME);

        builder = builder.with_flamegraph_layer(stack_filename);

        let flamegrapher = builder.init()?;

        if let Some(flamegrapher) = flamegrapher {
            return Ok(Some(flamegrapher.with_graph_file(graph_filename)?));
        } else {
            return Ok(flamegrapher);
        }
    }

    builder.init()
}
