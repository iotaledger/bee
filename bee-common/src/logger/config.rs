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

/// Default value for the color flag.
const DEFAULT_COLOR: bool = true;
/// Default name for an output.
const DEFAULT_NAME: &str = LOGGER_STDOUT_NAME;
/// Default log level for an output.
const DEFAULT_LEVEL: &str = "info";

/// Builder for a logger output configuration.
#[derive(Default, Deserialize)]
pub struct LoggerOutputConfigBuilder {
    // Name of an output.
    name: Option<String>,
    // Log level of an output.
    level: Option<String>,
}

impl LoggerOutputConfigBuilder {
    /// Creates a new builder for a logger output configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the name of a logger output.
    pub fn name(mut self, name: &str) -> Self {
        self.name.replace(name.to_string());
        self
    }

    /// Sets the level of a logger output.
    pub fn level(mut self, level: &str) -> Self {
        self.level.replace(level.to_string());
        self
    }

    /// Builds a logger output configuration.
    pub fn finish(self) -> LoggerOutputConfig {
        let level = match self.level.unwrap_or_else(|| DEFAULT_LEVEL.to_owned()).as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            _ => LevelFilter::Info,
        };

        LoggerOutputConfig {
            name: self.name.unwrap_or_else(|| DEFAULT_NAME.to_owned()),
            level,
        }
    }
}

/// Logger output configuration.
#[derive(Clone)]
pub struct LoggerOutputConfig {
    // Name of an output.
    pub(crate) name: String,
    // Log level of an output.
    pub(crate) level: LevelFilter,
}

/// Builder for a logger configuration.
#[derive(Default, Deserialize)]
pub struct LoggerConfigBuilder {
    // Color flag of the logger.
    color: Option<bool>,
    // Outputs of the logger.
    outputs: Option<Vec<LoggerOutputConfigBuilder>>,
}

impl LoggerConfigBuilder {
    /// Creates a new builder for a logger configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the color flag of a logger.
    pub fn color(mut self, color: bool) -> Self {
        self.color.replace(color);
        self
    }

    /// Sets the level of an output of a logger.
    pub fn level(&mut self, name: &str, level: String) {
        if let Some(outputs) = self.outputs.as_deref_mut() {
            if let Some(stdout) = outputs.iter_mut().find(|output| name == output.name.as_ref().unwrap()) {
                stdout.level.replace(level);
            }
        }
    }

    /// Builds a logger configuration.
    pub fn finish(self) -> LoggerConfig {
        let mut outputs = Vec::new();

        if let Some(content) = self.outputs {
            for output in content {
                outputs.push(output.finish());
            }
        }

        LoggerConfig {
            color: self.color.unwrap_or(DEFAULT_COLOR),
            outputs,
        }
    }
}

/// Logger configuration.
#[derive(Clone)]
pub struct LoggerConfig {
    // Color flag of the logger.
    pub(crate) color: bool,
    // Outputs of the logger.
    pub(crate) outputs: Vec<LoggerOutputConfig>,
}

impl LoggerConfig {
    // Creates a builder for a logger config.
    pub fn build() -> LoggerConfigBuilder {
        LoggerConfigBuilder::new()
    }
}
