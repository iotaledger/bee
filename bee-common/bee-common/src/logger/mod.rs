// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

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
    /// Creating output file failed.
    #[error("Creating output file failed.")]
    CreatingFileFailed,
    /// Initializing the logger backend failed.
    #[error("Initializing the logger backend failed.")]
    InitializationFailed,
}

macro_rules! log_format {
    ($target:expr, $level:expr, $message:expr, $target_width:expr, $level_width:expr) => {
        format_args!(
            "{} {:target_width$} {:level_width$} {}",
            crate::time::format(&crate::time::now_utc()),
            $target,
            $level,
            $message,
            target_width = $target_width,
            level_width = $level_width
        )
    };
}

/// Initializes a `fern` logger backend for the `log` crate.
///
/// # Arguments
///
/// * `config`  -   Logger configuration
pub fn logger_init(config: LoggerConfig) -> Result<(), Error> {
    let target_width = config.target_width;
    let level_width = config.level_width;

    let mut logger = Dispatch::new();

    for output in config.outputs {
        // Creates a logger dispatch for each output of the configuration.
        let mut dispatch = if output.color_enabled {
            let colors = ColoredLevelConfig::new()
                .trace(Color::BrightMagenta)
                .debug(Color::BrightBlue)
                .info(Color::BrightGreen)
                .warn(Color::BrightYellow)
                .error(Color::BrightRed);

            // Creates a logger dispatch with color support.
            Dispatch::new().format(move |out, message, record| {
                out.finish(log_format!(
                    record.target(),
                    colors.color(record.level()),
                    message,
                    target_width,
                    level_width
                ))
            })
        } else {
            // Creates a logger dispatch without color support.
            Dispatch::new().format(move |out, message, record| {
                out.finish(log_format!(
                    record.target(),
                    record.level(),
                    message,
                    target_width,
                    level_width
                ))
            })
        }
        .level(output.level_filter);

        if !output.target_filters.is_empty() {
            let target_filters = output.target_filters;
            dispatch = dispatch.filter(move |metadata| {
                let target = metadata.target().to_lowercase();
                target_filters.iter().any(|f| target.contains(f))
            });
        }

        if !output.target_exclusions.is_empty() {
            let target_exclusions = output.target_exclusions;
            dispatch = dispatch.filter(move |metadata| {
                let target = metadata.target().to_lowercase();
                !target_exclusions.iter().any(|f| target.contains(f))
            });
        }

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
