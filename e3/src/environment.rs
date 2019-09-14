//! Environment module.

use super::effect::Effect;
use super::supervisor::Notifier;

use crate::common::signal::{Signal, SignalRx};
use crate::common::waker::Waker;

use crate::constants::messages::*;
use crate::constants::BROADCAST_BUFFER_SIZE;

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use bus::Bus as Broadcaster;
use bus::BusReader as BroadcastReceiver;

use crossbeam_channel::Receiver;

use tokio::prelude::*;
use tokio::sync::watch;

/// Encapsulates information about an environment.
#[derive(Debug)]
pub struct EnvironmentInfo
{
    num_received_effects: usize,
    joined_entities: HashSet<String>,
    affecting_entities: HashSet<String>
}

impl EnvironmentInfo
{
    /// Returns the number of effects this environment has received.
    pub fn num_received_effects(&self) -> usize
    {
        self.num_received_effects
    }

    /// Returns the number of entities that joined this environment.
    pub fn num_joined_entities(&self) -> usize
    {
        self.joined_entities.len()
    }

    /// Returns the number of entities that affect this environment.
    pub fn num_affecting_entities(&self) -> usize
    {
        self.affecting_entities.len()
    }

    /// Returns whether the specified entity has joined this environment.
    pub fn has_joined(&self, entity_id: &str) -> bool
    {
        self.joined_entities.contains(entity_id)
    }

    /// Returns whether the specified entity affects this environment.
    pub fn affects(&self, entity_id: &str) -> bool
    {
        self.affecting_entities.contains(entity_id)
    }
}

pub(crate) struct Environment
{
    name: String,
    joined_entities: Arc<Mutex<HashMap<String, Waker>>>,
    affecting_entities: Arc<Mutex<HashMap<String, Notifier>>>,
    in_chan: Arc<Receiver<Effect>>,
    out_chan: Arc<Mutex<Broadcaster<Effect>>>,
    drop_notifier: Arc<Mutex<Signal>>,
    shutdown_listener: Arc<Mutex<SignalRx>>,
    terminate_chan: Arc<Mutex<(watch::Sender<bool>, watch::Receiver<bool>)>>,
    waker: Waker,
    num_received_effects: Arc<AtomicUsize>
}

impl Environment
{
    pub(crate) fn new(name: &str, effect_rx: Receiver<Effect>, term_rx: SignalRx) -> Self
    {
        Self {
            name: name.into(),
            joined_entities: share_mut!(HashMap::new()),
            affecting_entities: share_mut!(HashMap::new()),
            in_chan: share!(effect_rx),
            out_chan: share_mut!(Broadcaster::new(BROADCAST_BUFFER_SIZE)),
            drop_notifier: share_mut!(Signal::new()),
            shutdown_listener: share_mut!(term_rx),
            terminate_chan: share_mut!(watch::channel(false)),
            waker: Waker::new(),
            num_received_effects: share!(AtomicUsize::new(0))
        }
    }

    pub(crate) fn connect_input(
        &mut self,
        entity_id: &str,
        entity_rx: BroadcastReceiver<Effect>,
        entity_drop_rx: SignalRx
    ) -> Waker
    {
        let mut affecting_entities = unlock!(self.affecting_entities);

        if affecting_entities.contains_key(entity_id) {
            panic!(ENTITY_ALREADY_AFFECTS_ERROR);
        }

        affecting_entities.insert(
            entity_id.into(),
            Notifier {
                effect_rx: entity_rx,
                drop_rx: entity_drop_rx
            }
        );

        self.waker.clone()
    }

    pub(crate) fn disconnect_input(&mut self, entity_id: &str)
    {
        unlock!(self.affecting_entities).remove(entity_id);
    }

    pub(crate) fn get_connectors(&mut self) -> (BroadcastReceiver<Effect>, SignalRx)
    {
        let effect_rx = unlock!(self.out_chan).add_rx();
        let drop_rx = unlock!(self.drop_notifier).add_rx();

        (effect_rx, drop_rx)
    }

    pub(crate) fn connect_output(&mut self, entity_id: &str, entity_waker: Waker)
    {
        let mut joined_entities = unlock!(self.joined_entities);

        if joined_entities.contains_key(entity_id) {
            panic!(ENTITY_ALREADY_JOINED_ERROR);
        }

        joined_entities.insert(entity_id.into(), entity_waker);
    }

    pub(crate) fn disconnect_output(&mut self, entity_id: &str)
    {
        unlock!(self.joined_entities).remove(entity_id.into());
    }

    pub(crate) fn terminate(&self)
    {
        unlock!(self.drop_notifier).emit();

        #[cfg(debug_assertions)]
        println!(
            "environment \"{}\" notified joined entities about termination",
            self.name
        );

        let mut terminate = unlock_msg!(self.terminate_chan, UNLOCK_TERMINATION_CHAN_ERROR);
        terminate
            .0
            .broadcast(true)
            .expect(SEND_TERMINATION_SIGNAL_ERROR);
    }

    pub(crate) fn name(&self) -> &str
    {
        &self.name
    }

    pub(crate) fn get_waker(&self) -> Waker
    {
        self.waker.clone()
    }

    pub(crate) fn info(&self) -> EnvironmentInfo
    {
        use std::iter::FromIterator;

        let joined_entities = {
            let joined_entities = unlock!(self.joined_entities);
            HashSet::from_iter(joined_entities.keys().cloned())
        };

        let affecting_entities = {
            let affecting_entities = unlock!(self.affecting_entities);
            HashSet::from_iter(affecting_entities.keys().cloned())
        };

        EnvironmentInfo {
            num_received_effects: self.num_received_effects.load(Ordering::Relaxed),
            joined_entities,
            affecting_entities
        }
    }
}

impl Future for Environment
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), Self::Error>
    {
        self.waker.task.register();

        {
            let joined = unlock!(self.joined_entities);
            let mut affecting = unlock!(self.affecting_entities);
            let mut env_tx = unlock!(self.out_chan);

            let mut num_received = self.num_received_effects.load(Ordering::Acquire);
            let mut num = 0;

            loop {
                match self.in_chan.try_recv() {
                    Ok(effect) => {
                        num += 1;

                        #[cfg(debug_assertions)]
                        println!(
                            "environment \"{}\" received effect \"{:?}\" from supervisor ({})",
                            self.name,
                            effect,
                            num_received + num
                        );

                        env_tx.broadcast(effect);

                        // TODO: rework the following
                        // Wake all joined entities if half of the broadcaster
                        // buffer size is full
                        if num == BROADCAST_BUFFER_SIZE / 2 {
                            for (_, ent_waker) in joined.iter() {
                                ent_waker.task.notify();
                            }

                            num_received += num;
                            num = 0;
                        }
                    },
                    _ => break
                }
            } // end forwarding supervisor effects

            // Wake all joined entities to process the remaining effects buffered in the
            // broadcast channel
            for (_, ent_waker) in joined.iter() {
                ent_waker.task.notify();
            }

            num_received += num;
            num = 0;

            for (
                ent_id,
                Notifier {
                    effect_rx: ent_rx,
                    drop_rx: _
                }
            ) in affecting.iter_mut()
            {
                loop {
                    match ent_rx.try_recv() {
                        Ok(effect) => {
                            num += 1;

                            #[cfg(debug_assertions)]
                            println!(
                                "environment \"{}\" received effect '{:?}' from entity \"{}\" ({})",
                                self.name,
                                effect,
                                &ent_id[0..5],
                                num_received + num,
                            );
                        },
                        _ => break
                    }
                }
            }

            self.num_received_effects
                .store(num_received + num, Ordering::Release);
        } // end of this scope releases locks

        // Check for system-shutdown event
        match unlock!(self.shutdown_listener).0.poll() {
            Ok(Async::Ready(Some(is_shutdown_signal))) => {
                if is_shutdown_signal {
                    #[cfg(debug_assertions)]
                    println!("environment \"{}\" received shutdown signal", self.name);

                    return Ok(Async::Ready(()));
                }
            },
            _ => ()
        }

        // Check for environment-termination event
        match unlock!(self.terminate_chan).1.poll() {
            Ok(Async::Ready(Some(is_term_signal))) => {
                if is_term_signal {
                    #[cfg(debug_assertions)]
                    println!("environment \"{}\" received termination signal", self.name);

                    return Ok(Async::Ready(()));
                }
            },
            _ => ()
        }

        return Ok(Async::NotReady);
    }
}

impl Clone for Environment
{
    fn clone(&self) -> Self
    {
        Self {
            name: self.name.clone(),
            joined_entities: Arc::clone(&self.joined_entities),
            affecting_entities: Arc::clone(&self.affecting_entities),
            in_chan: Arc::clone(&self.in_chan),
            out_chan: Arc::clone(&self.out_chan),
            drop_notifier: Arc::clone(&self.drop_notifier),
            shutdown_listener: Arc::clone(&self.shutdown_listener),
            terminate_chan: Arc::clone(&self.terminate_chan),
            waker: self.waker.clone(),
            num_received_effects: Arc::clone(&self.num_received_effects)
        }
    }
}

impl Drop for Environment
{
    fn drop(&mut self)
    {
        #[cfg(debug_assertions)]
        println!("dropping environment \"{}\"", self.name());
    }
}
