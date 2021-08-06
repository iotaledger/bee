// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageId};

use crate::EventBus;

use backstage::{
    actor::{Actor, ActorError, EventDriven, SupervisorEvent, UnboundedTokioChannel},
    prelude::{ActorScopedRuntime, RegistryAccess, Res},
};

use std::marker::PhantomData;

// FIXME: import this from `bee-plugin` once it is merged.
pub struct MessageStoredEvent {}
pub struct MessageDeletedEvent {}
pub struct MissingMessageStoredEvent {}

pub enum StorageEvent {
    // FIXME: is the ID the right data?
    Cleanup { id: MessageId },
    Store { message: Message },
    StoreMissing { message: Message },
}

#[derive(Default)]
pub struct StorageWorker;

#[async_trait::async_trait]
impl Actor for StorageWorker {
    type Dependencies = Res<EventBus>;

    type Event = StorageEvent;

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
        println!("storage worker is up!");

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
        while let Some(event) = rt.next_event().await {
            match event {
                StorageEvent::Cleanup { id: _ } => {
                    // TODO
                    bus.dispatch(MessageDeletedEvent {});
                }
                StorageEvent::Store { message: _ } => {
                    // TODO
                    bus.dispatch(MessageStoredEvent {});
                }
                StorageEvent::StoreMissing { message: _ } => {
                    // TODO
                    bus.dispatch(MissingMessageStoredEvent {});
                }
            }
        }

        Ok(())
    }
}
