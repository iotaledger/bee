// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::mqtt::config::MqttConfig;

use log::warn;
use paho_mqtt as mqtt;
use thiserror::Error;

use std::time::Duration;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Mqtt operation failed: {0}.")]
    Mqtt(#[from] mqtt::errors::Error),
}

pub(crate) struct MqttManager {
    client: mqtt::AsyncClient,
}

impl MqttManager {
    pub(crate) async fn new(config: MqttConfig) -> Result<Self, Error> {
        let options = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(true)
            .finalize();

        let manager = Self {
            client: mqtt::AsyncClient::new(config.address().as_str())?,
        };

        manager.client.connect(options).await?;

        Ok(manager)
    }

    pub(crate) async fn send<P>(&self, topic: &'static str, payload: P)
    where
        P: Into<Vec<u8>>,
    {
        // TODO Send to all that registered to this topic
        if let Err(e) = self.client.publish(mqtt::Message::new(topic, payload, 0)).await {
            warn!("Publishing mqtt message on topic {} failed: {:?}.", topic, e);
        }
    }
}
