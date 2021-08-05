// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_packable::Packable;

use crate::EventBus;

use backstage::{
    actor::{Actor, ActorError, EventDriven, SupervisorEvent, UnboundedTokioChannel},
    prelude::{ActorScopedRuntime, RegistryAccess, Res},
};

use std::{collections::VecDeque, marker::PhantomData};

const RECENTLY_SEEN_MAX_LEN: usize = 10;

// FIXME: import this from `bee-plugin` once it is merged.
pub struct MessageParsedEvent {}
pub struct ParsingFailedEvent {}
pub struct MessageRejectedEvent {}

pub struct ParseEvent {
    pub bytes: Vec<u8>,
}

#[derive(Default)]
pub struct ParserWorker {
    recently_seen: VecDeque<Vec<u8>>,
}

impl ParserWorker {
    fn is_recent(&mut self, bytes: &[u8]) -> bool {
        for seen in &self.recently_seen {
            if bytes == seen {
                return true;
            }
        }

        if self.recently_seen.len() == RECENTLY_SEEN_MAX_LEN {
            self.recently_seen.pop_front().unwrap();
            self.recently_seen.push_back(bytes.to_vec());
        }

        false
    }
}

#[async_trait::async_trait]
impl Actor for ParserWorker {
    type Dependencies = Res<EventBus>;

    type Event = ParseEvent;

    type Channel = UnboundedTokioChannel<Self::Event>;

    async fn init<Reg: RegistryAccess + Send + Sync, Sup: EventDriven>(
        &mut self,
        _rt: &mut ActorScopedRuntime<Self, Reg, Sup>,
    ) -> Result<(), ActorError>
    where
        Self: Sized,
        Sup::Event: SupervisorEvent,
        <Sup::Event as SupervisorEvent>::Children: From<PhantomData<Self>>,
    {
        println!("parser is up!");

        Ok(())
    }

    async fn run<Reg: RegistryAccess + Send + Sync, Sup: EventDriven>(
        &mut self,
        rt: &mut ActorScopedRuntime<Self, Reg, Sup>,
        bus: Self::Dependencies,
    ) -> Result<(), ActorError>
    where
        Self: Sized,
        Sup::Event: SupervisorEvent,
        <Sup::Event as SupervisorEvent>::Children: From<PhantomData<Self>>,
    {
        while let Some(ParseEvent { bytes }) = rt.next_event().await {
            if !self.is_recent(&bytes) {
                match bee_message::Message::unpack_from_slice(&bytes) {
                    Ok(_message) => {
                        // FIXME: figure out the remaining validation steps.
                        bus.dispatch(MessageParsedEvent {});
                    }
                    Err(err) => match err {
                        bee_packable::UnpackError::Packable(_err) => {
                            bus.dispatch(MessageRejectedEvent {});
                        }
                        bee_packable::UnpackError::Unpacker(_err) => {
                            bus.dispatch(ParsingFailedEvent {});
                        }
                    },
                }
            }
        }

        Ok(())
    }
}
