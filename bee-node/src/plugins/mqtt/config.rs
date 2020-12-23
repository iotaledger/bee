// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

const DEFAULT_ADDRESS: &str = "tcp://localhost:1883";

#[derive(Default, Deserialize)]
pub struct MqttConfigBuilder {
    address: Option<String>,
}

impl MqttConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> MqttConfig {
        MqttConfig {
            address: self.address.unwrap_or_else(|| DEFAULT_ADDRESS.to_owned()),
        }
    }
}

#[derive(Clone)]
pub struct MqttConfig {
    address: String,
}

impl MqttConfig {
    pub fn address(&self) -> &String {
        &self.address
    }
}
