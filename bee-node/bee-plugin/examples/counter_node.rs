// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_event_bus::EventBus;
use bee_logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
use bee_plugin::{event::*, hotloading::Hotloader, UniqueId};

use tokio::time::{sleep, Duration};

use std::{io::ErrorKind, sync::Arc};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger_init(LoggerConfig::build().output(LoggerOutputConfigBuilder::new()).finish())?;

    match tokio::fs::create_dir("./plugins").await {
        Err(err) if err.kind() != ErrorKind::AlreadyExists => return Err(err.into()),
        _ => (),
    }

    let event_bus = Arc::new(EventBus::<UniqueId>::new());

    let hotloader = Hotloader::new("./plugins", Arc::clone(&event_bus));

    let handle = tokio::spawn(async move { hotloader.run().await });

    {
        let event_bus = Arc::clone(&event_bus);
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(1)).await;
                event_bus.dispatch(MessageParsedEvent {})
            }
        });
    }

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(1)).await;
            event_bus.dispatch(MessageRejectedEvent {})
        }
    });

    handle.await??;

    Ok(())
}
