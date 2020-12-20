// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod config;
mod topics;

use topics::*;

use bee_common_pt2::{
    event::Bus,
    node::{Node, ResHandle},
    worker::Worker,
};
use bee_protocol::event::{LatestMilestoneChanged, LatestSolidMilestoneChanged};

use async_trait::async_trait;
use log::{error, warn};
use paho_mqtt as mqtt;

use std::{convert::Infallible, time::Duration};

#[derive(Default)]
pub struct Mqtt;

fn send_message<P>(client: &ResHandle<mqtt::AsyncClient>, topic: &'static str, payload: P)
where
    P: Into<Vec<u8>>,
{
    if let Err(e) = client.publish(mqtt::Message::new(topic, payload, 0)).wait() {
        warn!("Publishing mqtt message on topic {} failed: {:?}.", topic, e);
    }
}

#[async_trait]
impl<N: Node> Worker<N> for Mqtt {
    type Config = ();
    type Error = Infallible;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let bus = node.resource::<Bus>();

        // TODO conf
        match mqtt::AsyncClient::new("tcp://localhost:1883") {
            Ok(client) => {
                node.register_resource(client);
                let client = node.resource::<mqtt::AsyncClient>();

                let conn_opts = mqtt::ConnectOptionsBuilder::new()
                    .keep_alive_interval(Duration::from_secs(20))
                    .clean_session(true)
                    .finalize();

                match client.connect(conn_opts).wait() {
                    Ok(_) => {
                        // TODO log connected

                        let _client = client.clone();
                        bus.add_listener::<Self, _, _>(move |_latest_milestone: &LatestMilestoneChanged| {
                            send_message(&_client, TOPIC_MILESTONES_LATEST, "");
                        });

                        let _client = client.clone();
                        bus.add_listener::<Self, _, _>(move |_latest_solid_milestone: &LatestSolidMilestoneChanged| {
                            send_message(&_client, TOPIC_MILESTONES_SOLID, "");
                        });

                        // let _client = client.clone();
                        // bus.add_listener::<Self, _, _>(move |_: &_| {
                        //     send_message(&_client, _TOPIC_MESSAGES, _);
                        // });
                        //
                        // let _client = client.clone();
                        // bus.add_listener::<Self, _, _>(move |_: &_| {
                        //     send_message(&_client, _TOPIC_MESSAGES_REFERENCED, _);
                        // });
                        //
                        // let _client = client.clone();
                        // bus.add_listener::<Self, _, _>(move |_: &_| {
                        //     send_message(&_client, _TOPIC_MESSAGES_INDEXATION, _);
                        // });
                        //
                        // let _client = client.clone();
                        // bus.add_listener::<Self, _, _>(move |_: &_| {
                        //     send_message(&_client, _TOPIC_MESSAGES_METADATA, _);
                        // });
                        //
                        // let _client = client.clone();
                        // bus.add_listener::<Self, _, _>(move |_: &_| {
                        //     send_message(&_client, _TOPIC_OUTPUTS, _);
                        // });
                        //
                        // let _client = client.clone();
                        // bus.add_listener::<Self, _, _>(move |_: &_| {
                        //     send_message(&_client, _TOPIC_ADDRESSES_OUTPUTS, _);
                        // });
                        //
                        // let _client = client.clone();
                        // bus.add_listener::<Self, _, _>(move |_: &_| {
                        //     send_message(&_client, _TOPIC_ADDRESSES_ED25519_OUTPUT, _);
                        // });
                    }
                    Err(e) => {
                        error!("Connecting mqtt client failed {:?}.", e);
                    }
                }
            }
            Err(e) => {
                error!("Creating mqtt client failed {:?}.", e);
            }
        }

        Ok(Self::default())
    }

    async fn stop(self, _node: &mut N) -> Result<(), Self::Error> {
        // if let Some(client) = node.remove_resource::<mqtt::AsyncClient>() {
        //     client.disconnect(None).wait().unwrap();
        // }

        Ok(())
    }
}
