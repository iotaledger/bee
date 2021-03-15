// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

const DEFAULT_PORT: u16 = 1883;

#[derive(Clone)]
pub struct MqttConfig {
    pub(crate) port: u16,
}

#[derive(Default, Deserialize)]
pub struct MqttConfigBuilder {
    port: Option<u16>,
}

impl MqttConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> MqttConfig {
        MqttConfig {
            port: self.port.unwrap_or(DEFAULT_PORT),
        }
    }
}
