// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_event_bus::EventBus;
use bee_logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
use bee_plugin::{event::*, hotloader::PluginHotloader, UniqueId};

use tokio::time::{sleep, Duration};

use std::{io::ErrorKind, sync::Arc};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger_init(LoggerConfig::build().output(LoggerOutputConfigBuilder::new()).finish())?;

    if let Err(e) = tokio::fs::create_dir("./plugins").await {
        if e.kind() != ErrorKind::AlreadyExists {
            return Err(e.into());
        }
    }

    let bus = Arc::new(EventBus::<UniqueId>::new());
    let hotloader = PluginHotloader::new("./plugins", bus.clone());
    let handle = tokio::spawn(async move { hotloader.run().await });

    {
        let bus = bus.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(1)).await;
                bus.dispatch(MessageParsedEvent {})
            }
        });
    }

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(1)).await;
            bus.dispatch(MessageRejectedEvent {})
        }
    });

    handle.await??;

    Ok(())
}
