// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::LOGGER_STDOUT_NAME;

use log::LevelFilter;

use std::borrow::Cow;

/// Default name for an output.
const DEFAULT_OUTPUT_NAME: &str = LOGGER_STDOUT_NAME;
/// Default log level for an output.
const DEFAULT_OUTPUT_LEVEL_FILTER: LevelFilter = LevelFilter::Info;
/// Default value for the color flag.
const DEFAULT_COLOR_ENABLED: bool = true;
/// Default value for the target width.
const DEFAULT_TARGET_WIDTH: usize = 42;
/// Default value for the level width.
const DEFAULT_LEVEL_WIDTH: usize = 5;

/// Builder for a logger output configuration.
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct LoggerOutputConfigBuilder {
    /// Name of an output file, or `stdout` for standard output.
    name: Option<String>,
    /// Log level filter of an output.
    level_filter: Option<LevelFilter>,
    /// Log target filters of an output.
    target_filters: Option<Vec<String>>,
    /// Log target exclusions of an output.
    target_exclusions: Option<Vec<String>>,
}

impl LoggerOutputConfigBuilder {
    /// Creates a new `LoggerOutputConfigBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the name of a `LoggerOutputConfigBuilder`.
    pub fn name<'a>(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name.replace(name.into().into_owned());
        self
    }

    /// Sets the level of a `LoggerOutputConfigBuilder`.
    pub fn level_filter(mut self, level: LevelFilter) -> Self {
        self.level_filter.replace(level);
        self
    }

    /// Sets a collection of target filters of a `LoggerOutputConfigBuilder`.
    /// A message is logged only if one of the filters is part of the log's metadata target.
    pub fn target_filters(mut self, target_filters: &[&str]) -> Self {
        self.target_filters = Some(target_filters.iter().map(|f| f.to_string()).collect::<Vec<String>>());
        self
    }

    /// Sets a collection of target exclusions of a `LoggerOutputConfigBuilder`.
    /// A message is not logged if one of the exclusions is part of the log's metadata target.
    pub fn target_exclusions(mut self, target_exclusions: &[&str]) -> Self {
        self.target_exclusions = Some(target_exclusions.iter().map(|f| f.to_string()).collect::<Vec<String>>());
        self
    }

    /// Finishes a `LoggerOutputConfigBuilder` into a `LoggerOutputConfig`.
    pub fn finish(self) -> LoggerOutputConfig {
        LoggerOutputConfig {
            name: self.name.unwrap_or_else(|| DEFAULT_OUTPUT_NAME.to_owned()),
            level_filter: self.level_filter.unwrap_or(DEFAULT_OUTPUT_LEVEL_FILTER),
            target_filters: self
                .target_filters
                .unwrap_or_else(Vec::new)
                .iter()
                .map(|f| f.to_lowercase())
                .collect(),
            target_exclusions: self
                .target_exclusions
                .unwrap_or_else(Vec::new)
                .iter()
                .map(|f| f.to_lowercase())
                .collect(),
        }
    }
}

/// Logger output configuration.
#[derive(Clone)]
pub struct LoggerOutputConfig {
    /// Name of an output file, or `stdout` for standard output.
    pub(crate) name: String,
    /// Log level of an output.
    pub(crate) level_filter: LevelFilter,
    /// Log target filters of the output.
    pub(crate) target_filters: Vec<String>,
    /// Log target exclusions of the output.
    pub(crate) target_exclusions: Vec<String>,
}

/// Builder for a logger configuration.
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct LoggerConfigBuilder {
    /// Color flag of the logger.
    color_enabled: Option<bool>,
    /// Width of the target section of a log.
    target_width: Option<usize>,
    /// Width of the level section of a log.
    level_width: Option<usize>,
    /// Outputs of the logger.
    outputs: Option<Vec<LoggerOutputConfigBuilder>>,
}

impl LoggerConfigBuilder {
    /// Sets the color flag of a `LoggerConfigBuilder`.
    pub fn color_enabled(mut self, color: bool) -> Self {
        self.color_enabled.replace(color);
        self
    }

    /// Sets the target width of a `LoggerConfigBuilder`.
    pub fn target_width(mut self, width: usize) -> Self {
        self.target_width.replace(width);
        self
    }

    /// Sets the target width of a `LoggerConfigBuilder`.
    pub fn level_width(mut self, width: usize) -> Self {
        self.level_width.replace(width);
        self
    }

    /// Adds an output builder to a `LoggerConfigBuilder`.
    pub fn output(mut self, output: LoggerOutputConfigBuilder) -> Self {
        self.outputs.get_or_insert_with(Vec::new).push(output);
        self
    }

    /// Sets the level of a `LoggerConfigBuilder`.
    pub fn level<'a>(&mut self, name: impl Into<Cow<'a, str>>, level: LevelFilter) {
        let name = name.into();

        if let Some(outputs) = self.outputs.as_deref_mut() {
            if let Some(stdout) = outputs.iter_mut().find(|output| match output.name.as_ref() {
                Some(output_name) => output_name[..] == name,
                None => false,
            }) {
                stdout.level_filter.replace(level);
            }
        }
    }

    /// Finishes a `LoggerConfigBuilder` into a `LoggerConfig`.
    pub fn finish(self) -> LoggerConfig {
        LoggerConfig {
            color_enabled: self.color_enabled.unwrap_or(DEFAULT_COLOR_ENABLED),
            target_width: self.target_width.unwrap_or(DEFAULT_TARGET_WIDTH),
            level_width: self.level_width.unwrap_or(DEFAULT_LEVEL_WIDTH),
            outputs: self
                .outputs
                .map(|outputs| outputs.into_iter().map(LoggerOutputConfigBuilder::finish).collect())
                .unwrap_or_default(),
        }
    }
}

/// Logger configuration.
#[derive(Clone)]
pub struct LoggerConfig {
    /// Color flag of the logger.
    pub(crate) color_enabled: bool,
    /// Width of the target section of a log.
    pub(crate) target_width: usize,
    /// Width of the level section of a log.
    pub(crate) level_width: usize,
    /// Outputs of the logger.
    pub(crate) outputs: Vec<LoggerOutputConfig>,
}

impl LoggerConfig {
    /// Creates a new `LoggerConfigBuilder`.
    pub fn build() -> LoggerConfigBuilder {
        LoggerConfigBuilder::default()
    }
}
