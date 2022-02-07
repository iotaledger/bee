// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

const DEFAULT_ADDRESS: &str = "tcp://localhost:1883";

#[derive(Default, Deserialize, PartialEq)]
pub struct MqttConfigBuilder {
    address: Option<String>,
}

impl MqttConfigBuilder {
    /// Creates a new [`MqttConfigBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a new [`MqttConfig`], consuming the [`MqttConfigBuilder`].
    pub fn finish(self) -> MqttConfig {
        MqttConfig {
            address: self.address.unwrap_or_else(|| DEFAULT_ADDRESS.to_owned()),
        }
    }
}

/// MQTT plugin configuration.
#[derive(Clone)]
pub struct MqttConfig {
    address: String,
}

impl MqttConfig {
    /// Returnns the address of the MQTT broker.
    pub fn address(&self) -> &String {
        &self.address
    }
}
