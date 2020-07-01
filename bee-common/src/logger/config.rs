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

use crate::logger::LOGGER_STDOUT_NAME;

use log::LevelFilter;
use serde::Deserialize;

use std::borrow::Cow;

/// Default value for the color flag.
const DEFAULT_COLOR_ENABLED: bool = true;
/// Default name for an output.
const DEFAULT_OUTPUT_NAME: &str = LOGGER_STDOUT_NAME;
/// Default log level for an output.
const DEFAULT_OUTPUT_LEVEL: LevelFilter = LevelFilter::Info;

/// Builder for a logger output configuration.
#[derive(Default, Deserialize)]
pub struct LoggerOutputConfigBuilder {
    /// Name of an output file, or `stdout` for standard output.
    name: Option<String>,
    /// Log level of an output.
    level: Option<LevelFilter>,
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
    pub fn level(mut self, level: LevelFilter) -> Self {
        self.level.replace(level);
        self
    }

    /// Builds a logger output configuration.
    pub fn finish(self) -> LoggerOutputConfig {
        LoggerOutputConfig {
            name: self.name.unwrap_or_else(|| DEFAULT_OUTPUT_NAME.to_owned()),
            level: self.level.unwrap_or(DEFAULT_OUTPUT_LEVEL),
        }
    }
}

/// Logger output configuration.
#[derive(Clone)]
pub struct LoggerOutputConfig {
    /// Name of an output file, or `stdout` for standard output.
    pub(crate) name: String,
    /// Log level of an output.
    pub(crate) level: LevelFilter,
}

/// Builder for a logger configuration.
#[derive(Default, Deserialize)]
pub struct LoggerConfigBuilder {
    /// Color flag of the logger.
    color_enabled: Option<bool>,
    /// Outputs of the logger.
    outputs: Option<Vec<LoggerOutputConfigBuilder>>,
}

impl LoggerConfigBuilder {
    /// Sets the color flag of a logger.
    pub fn color_enabled(mut self, color: bool) -> Self {
        self.color_enabled.replace(color);
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
                stdout.level.replace(level);
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
            outputs,
        }
    }
}

/// Logger configuration.
#[derive(Clone)]
pub struct LoggerConfig {
    /// Color flag of the logger.
    pub(crate) color_enabled: bool,
    /// Outputs of the logger.
    pub(crate) outputs: Vec<LoggerOutputConfig>,
}

impl LoggerConfig {
    // Creates a builder for a logger config.
    pub fn build() -> LoggerConfigBuilder {
        LoggerConfigBuilder::default()
    }
}
