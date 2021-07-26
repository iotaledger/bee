// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A logger backend for the `log` crate.

#![warn(missing_docs)]

mod config;

pub use config::{LoggerConfig, LoggerConfigBuilder, LoggerOutputConfig, LoggerOutputConfigBuilder};

use chrono::Local;
use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};
use thiserror::Error;

/// Name of the standard output. [`Test`]
pub const LOGGER_STDOUT_NAME: &str = "stdout";

/// Errors occuring when initialising a logger backend.
#[derive(Error, Debug)]
pub enum Error {
    /// Creating output file failed.
    #[error("Creating output file failed.")]
    CreatingFileFailed,
    /// Initialising the logger backend failed.
    #[error("Initialising the logger backend failed.")]
    InitialisationFailed,
}

macro_rules! log_format {
    ($target:expr, $level:expr, $message:expr, $target_width:expr, $level_width:expr) => {
        format_args!(
            "{} {:target_width$} {:level_width$} {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            $target,
            $level,
            $message,
            target_width = $target_width,
            level_width = $level_width
        )
    };
}

/// Initialises a `fern` logger backend for the `log` crate.
///
/// # Arguments
///
/// * `config`  -   Logger configuration
pub fn logger_init(config: LoggerConfig) -> Result<(), Error> {
    let target_width = config.target_width;
    let level_width = config.level_width;

    let mut logger = if config.color_enabled {
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
    };

    for output in config.outputs {
        // Creates a logger dispatch for each output of the configuration.
        let mut dispatch = Dispatch::new().level(output.level_filter);

        if !output.target_filters.is_empty() {
            let target_filters = output.target_filters;
            // Filter targets according to configuration.
            dispatch = dispatch.filter(move |metadata| {
                let target = metadata.target().to_lowercase();
                target_filters.iter().any(|f| target.contains(f))
            });
        }

        if !output.target_exclusions.is_empty() {
            let target_exclusions = output.target_exclusions;
            // Exclude targets according to configuration.
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

    logger.apply().map_err(|_| Error::InitialisationFailed)?;

    Ok(())
}
