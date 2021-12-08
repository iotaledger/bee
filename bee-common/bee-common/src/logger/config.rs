// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::logger::LOGGER_STDOUT_NAME;

use log::LevelFilter;
use serde::Deserialize;

use std::borrow::Cow;

/// Default value for the color flag.
const DEFAULT_COLOR_ENABLED: bool = true;
/// Default value for the target width.
const DEFAULT_TARGET_WIDTH: usize = 42;
/// Default value for the level width.
const DEFAULT_LEVEL_WIDTH: usize = 5;
/// Default name for an output.
const DEFAULT_OUTPUT_NAME: &str = LOGGER_STDOUT_NAME;
/// Default log level for an output.
const DEFAULT_OUTPUT_LEVEL: LevelFilter = LevelFilter::Info;

/// Builder for a logger output configuration.
#[derive(Default, Deserialize)]
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
    /// Creates a new builder for a logger output configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the name of a logger output.
    pub fn name<'a>(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name.replace(name.into().into_owned());
        self
    }

    /// Sets the level of a logger output.
    pub fn level_filter(mut self, level: LevelFilter) -> Self {
        self.level_filter.replace(level);
        self
    }

    /// Sets a collection of filters of a logger output.
    /// A message is logged only if one of the filters is part of the log's metadata target.
    pub fn target_filters(mut self, target_filters: &[&str]) -> Self {
        self.target_filters = Some(target_filters.iter().map(ToString::to_string).collect::<Vec<_>>());
        self
    }

    /// Sets a collection of exclusions of a logger output.
    /// A message is logged only if one of the exclusions is *not* part of the log's metadata target.
    pub fn target_exclusions(mut self, target_exclusions: &[&str]) -> Self {
        self.target_exclusions = Some(target_exclusions.iter().map(ToString::to_string).collect::<Vec<_>>());
        self
    }

    /// Builds a logger output configuration.
    pub fn finish(self) -> LoggerOutputConfig {
        LoggerOutputConfig {
            name: self.name.unwrap_or_else(|| DEFAULT_OUTPUT_NAME.to_owned()),
            level_filter: self.level_filter.unwrap_or(DEFAULT_OUTPUT_LEVEL),
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

impl LoggerOutputConfig {
    /// Returns the name of the output file, or `stdout` for standard output.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the log level of the output.
    pub fn level_filter(&self) -> LevelFilter {
        self.level_filter
    }

    /// Returns the target filters of the output.
    pub fn target_filters(&self) -> &[String] {
        &self.target_filters
    }

    /// Returns the target exclusions of the output.
    pub fn target_exclusions(&self) -> &[String] {
        &self.target_exclusions
    }
}

/// Builder for a logger configuration.
#[derive(Default, Deserialize)]
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
    /// Sets the color flag of a logger.
    pub fn color_enabled(mut self, color: bool) -> Self {
        self.color_enabled.replace(color);
        self
    }

    /// Sets the target width.
    pub fn with_target_width(mut self, width: usize) -> Self {
        self.target_width.replace(width);
        self
    }

    /// Sets the target width.
    pub fn with_level_width(mut self, width: usize) -> Self {
        self.level_width.replace(width);
        self
    }

    /// Adds an output builder to the logger builder.
    pub fn with_output(mut self, output: LoggerOutputConfigBuilder) -> Self {
        self.outputs.get_or_insert_with(Vec::new).push(output);
        self
    }

    /// Sets the level of an output of a logger.
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

    /// Builds a logger configuration.
    pub fn finish(self) -> LoggerConfig {
        let outputs = self
            .outputs
            .map(|os| os.into_iter().map(|o| o.finish()).collect())
            .unwrap_or_default();

        LoggerConfig {
            color_enabled: self.color_enabled.unwrap_or(DEFAULT_COLOR_ENABLED),
            target_width: self.target_width.unwrap_or(DEFAULT_TARGET_WIDTH),
            level_width: self.level_width.unwrap_or(DEFAULT_LEVEL_WIDTH),
            outputs,
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
    /// Creates a builder for a logger config.
    pub fn build() -> LoggerConfigBuilder {
        LoggerConfigBuilder::default()
    }

    /// Returns the color flag of the `LoggerConfig`.
    pub fn color_enabled(&self) -> bool {
        self.color_enabled
    }

    /// Returns the width of the target section of the `LoggerConfig`.
    pub fn target_width(&self) -> usize {
        self.target_width
    }

    /// Returns the width of the level section of the `LoggerConfig`.
    pub fn level_width(&self) -> usize {
        self.level_width
    }

    /// Returns the outputs of the `LoggerConfig`.
    pub fn outputs(&self) -> &[LoggerOutputConfig] {
        &self.outputs
    }
}
