// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct MqttConfigBuilder {}

impl MqttConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> MqttConfig {
        MqttConfig {}
    }
}

#[derive(Clone)]
pub struct MqttConfig {}

impl MqttConfig {
    pub fn build() -> MqttConfigBuilder {
        MqttConfigBuilder::new()
    }
}
