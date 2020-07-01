// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

//! A logger backend for the `log` crate.

mod config;

pub use config::{LoggerConfig, LoggerConfigBuilder, LoggerOutputConfig, LoggerOutputConfigBuilder};

use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};
use thiserror::Error;

/// Name of the standard output.
pub const LOGGER_STDOUT_NAME: &str = "stdout";

/// Error occuring when initializing a logger backend.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Creating output file failed.")]
    CreatingFileFailed,
    #[error("Initializing the logger backend failed.")]
    InitializationFailed,
}

macro_rules! log_format {
    ($record:expr, $level:expr, $message:expr) => {
        format_args!(
            "{}[{}][{}] {}",
            chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
            $record,
            $level,
            $message
        )
    };
}

/// Initializes a `fern` logger backend for the `log` crate.
///
/// # Arguments
///
/// * `config`  -   Logger configuration
pub fn logger_init(config: LoggerConfig) -> Result<(), Error> {
    let mut logger = if config.color_enabled {
        let colors = ColoredLevelConfig::new()
            .trace(Color::BrightMagenta)
            .debug(Color::BrightBlue)
            .info(Color::BrightGreen)
            .warn(Color::BrightYellow)
            .error(Color::BrightRed);

        // Creates a logger dispatch with color support.
        Dispatch::new().format(move |out, message, record| {
            out.finish(log_format!(record.target(), colors.color(record.level()), message))
        })
    } else {
        // Creates a logger dispatch without color support.
        Dispatch::new()
            .format(move |out, message, record| out.finish(log_format!(record.target(), record.level(), message)))
    };

    for output in config.outputs {
        // Creates a logger dispatch for each output of the configuration.
        let mut dispatch = Dispatch::new().level(output.level);

        // Special case for the standard output.
        dispatch = if output.name == LOGGER_STDOUT_NAME {
            dispatch.chain(std::io::stdout())
        } else {
            dispatch.chain(fern::log_file(output.name).map_err(|_| Error::CreatingFileFailed)?)
        };

        logger = logger.chain(dispatch);
    }

    logger.apply().map_err(|_| Error::InitializationFailed)?;

    Ok(())
}
