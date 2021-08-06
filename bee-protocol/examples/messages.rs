// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_event_bus::EventBus;
use bee_protocol::*;

use backstage::prelude::*;

use std::sync::Arc;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    RuntimeScope::<ActorRegistry>::launch(|scope| {
        Box::pin(async move {
            let bus = scope
                .add_resource(Arc::new(EventBus::<'static, std::any::TypeId>::new()))
                .await;

            let _parser = scope.spawn_actor_unsupervised(ParserWorker::default()).await?;

            let storage = scope.spawn_actor_unsupervised(StorageWorker::default()).await?;

            bus.add_listener::<MessageParsedEvent, _, _>(move |event: &MessageParsedEvent| {
                println!("MessageParsedEvent triggered!");
                if let Err(err) = storage.send(StorageEvent::Store {
                    message: event.message.clone(),
                }) {
                    eprintln!("Error calling `store`: {}", err);
                }
            });
            bus.add_listener::<ParsingFailedEvent, _, _>(|_event: &ParsingFailedEvent| {
                println!("ParsingFailedEvent triggered!")
            });
            bus.add_listener::<MessageRejectedEvent, _, _>(|_event: &MessageRejectedEvent| {
                println!("MessageRejectedEvent triggered!")
            });
            bus.add_listener::<MessageStoredEvent, _, _>(|_event: &MessageStoredEvent| {
                println!("MessageStoredEvent triggered!")
            });

            let valid_bytes = vec![
                1, 2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3,
                238, 142, 220, 195, 72, 99, 77, 135, 73, 71, 196, 160, 101, 213, 130, 203, 214, 96, 245, 30, 3, 44, 37,
                103, 128, 55, 240, 155, 139, 220, 142, 178, 216, 230, 192, 191, 209, 104, 112, 20, 2, 0, 0, 0, 1, 104,
                0, 0, 0, 8, 0, 0, 0, 0, 32, 0, 0, 0, 132, 27, 114, 220, 115, 116, 126, 193, 10, 134, 212, 173, 149,
                101, 177, 183, 239, 215, 196, 68, 91, 60, 110, 222, 214, 229, 233, 139, 78, 192, 242, 72, 64, 0, 0, 0,
                153, 128, 64, 149, 20, 34, 176, 142, 218, 58, 195, 204, 46, 40, 206, 2, 5, 166, 147, 196, 253, 226,
                199, 30, 119, 83, 20, 169, 249, 80, 123, 20, 163, 123, 208, 238, 69, 191, 198, 110, 105, 107, 184, 244,
                12, 51, 64, 199, 121, 8, 14, 248, 38, 118, 144, 2, 133, 4, 126, 169, 122, 117, 124, 134, 0, 0, 0, 0, 0,
                0, 0, 0, 145, 167, 69, 239, 139, 44, 177, 36, 175, 85, 127, 123, 121, 5, 53, 252, 47, 72, 99, 133, 46,
                48, 76, 67, 166, 136, 216, 171, 49, 120, 150, 197, 94, 234, 36, 251, 59, 102, 43, 196, 54, 55, 138,
                254, 248, 226, 27, 75, 64, 65, 70, 179, 143, 249, 27, 85, 91, 169, 46, 237, 98, 213, 205, 27,
            ];

            let mut invalid_bytes = valid_bytes.clone();
            invalid_bytes[0] = 2;

            let missing_bytes = valid_bytes[0..valid_bytes.len() / 2].to_vec();

            scope
                .send_actor_event::<ParserWorker>(ParseEvent { bytes: valid_bytes })
                .await?;
            scope
                .send_actor_event::<ParserWorker>(ParseEvent { bytes: invalid_bytes })
                .await?;
            scope
                .send_actor_event::<ParserWorker>(ParseEvent { bytes: missing_bytes })
                .await?;

            Ok(())
        })
    })
    .await?;

    Ok(())
}
