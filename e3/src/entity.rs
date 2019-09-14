use super::effect::Effect;
use super::supervisor::Notifier;

use crate::common::signal::Signal;
use crate::common::signal::SignalRx;
use crate::common::waker::Waker;

use crate::constants::messages::*;
use crate::constants::BROADCAST_BUFFER_SIZE;

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use bus::Bus as Broadcaster;
use bus::BusReader as BroadcastReceiver;

use tokio::io;
use tokio::prelude::*;
use tokio::sync::watch;

use uuid::Uuid;

/// Provides contextual information for the processing entity about the given effect.  
#[derive(Debug)]
pub struct Context<'a>
{
    /// Stores the origin of the effect.
    pub origin: &'a str
}

/// A trait defining an entity.
pub trait Entity: Send
{
    /// Processes an effect and yields another.
    fn process(&mut self, effect: Effect, context: Context<'_>) -> Effect;
}

/// Encapsulates information about an entity.
#[derive(Debug)]
pub struct EntityInfo
{
    num_received_effects: usize,
    joined_environments: HashSet<String>,
    affecting_environments: HashSet<String>
}

impl EntityInfo
{
    /// Returns the number of effects this entity has received.
    pub fn num_received_effects(&self) -> usize
    {
        self.num_received_effects
    }

    /// Returns the number of environments this entity has joined.
    pub fn num_joined_environments(&self) -> usize
    {
        self.joined_environments.len()
    }

    /// Returns the number of environments this entity affects.
    pub fn num_affecting_environments(&self) -> usize
    {
        self.affecting_environments.len()
    }

    /// Returns whether this entity has joined the specified environment.
    pub fn has_joined_environment(&self, environment_name: &str) -> bool
    {
        self.joined_environments.contains(environment_name)
    }

    /// Returns whether this entity affects the specified environment.
    pub fn affects_environment(&self, environment_name: &str) -> bool
    {
        self.affecting_environments.contains(environment_name)
    }
}

pub(crate) struct EntityHarness
{
    id: String,
    joined_environments: Arc<Mutex<HashMap<String, Notifier>>>,
    affected_environments: Arc<Mutex<HashMap<String, Waker>>>,
    out_chan: Arc<Mutex<Broadcaster<Effect>>>,
    drop_notifier: Arc<Mutex<Signal>>,
    shutdown_listener: Arc<Mutex<SignalRx>>,
    terminate_chan: Arc<Mutex<(watch::Sender<bool>, watch::Receiver<bool>)>>,
    waker: Waker,
    num_received_effects: Arc<AtomicUsize>,
    entity: Arc<Mutex<Box<dyn Entity>>>
}

impl EntityHarness
{
    pub(crate) fn new(entity: Box<dyn Entity>, shutdown_rx: SignalRx) -> Self
    {
        Self {
            id: Uuid::new_v4().to_string(),
            joined_environments: share_mut!(HashMap::new()),
            affected_environments: share_mut!(HashMap::new()),
            out_chan: share_mut!(Broadcaster::new(BROADCAST_BUFFER_SIZE)),
            drop_notifier: share_mut!(Signal::new()),
            shutdown_listener: share_mut!(shutdown_rx),
            terminate_chan: share_mut!(watch::channel(false)),
            waker: Waker::new(),
            num_received_effects: share!(AtomicUsize::new(0)),
            entity: share_mut!(entity)
        }
    }

    pub(crate) fn connect_input(
        &mut self,
        environment_name: &str,
        environment_rx: BroadcastReceiver<Effect>,
        environment_drop_rx: SignalRx
    ) -> Waker
    {
        let mut joined_environments = unlock!(self.joined_environments);

        if joined_environments.contains_key(environment_name) {
            panic!(ENTITY_ALREADY_JOINED_ERROR);
        }

        joined_environments.insert(
            environment_name.into(),
            Notifier {
                effect_rx: environment_rx,
                drop_rx: environment_drop_rx
            }
        );

        self.waker.clone()
    }

    pub(crate) fn disconnect_input(&mut self, environment_name: &str)
    {
        unlock!(self.joined_environments).remove(environment_name);
    }

    pub(crate) fn get_connectors(&mut self) -> (BroadcastReceiver<Effect>, SignalRx)
    {
        let effect_rx = unlock!(self.out_chan).add_rx();
        let drop_rx = unlock!(self.drop_notifier).add_rx();

        (effect_rx, drop_rx)
    }

    pub(crate) fn connect_output(&mut self, environment_name: &str, environment_waker: Waker)
    {
        let mut affected_environments = unlock!(self.affected_environments);

        if affected_environments.contains_key(environment_name) {
            panic!(ENTITY_ALREADY_AFFECTS_ERROR);
        }

        affected_environments.insert(environment_name.into(), environment_waker);
    }

    pub(crate) fn disconnect_output(&mut self, environment_name: &str)
    {
        unlock!(self.affected_environments).remove(environment_name);
    }

    pub(crate) fn terminate(&self)
    {
        unlock!(self.drop_notifier).emit();

        #[cfg(debug_assertions)]
        println!(
            "entity \"{}\" notified affected environments about termination",
            &self.id[0..5]
        );

        let mut terminate = unlock_msg!(self.terminate_chan, UNLOCK_TERMINATION_CHAN_ERROR);
        terminate
            .0
            .broadcast(true)
            .expect(SEND_TERMINATION_SIGNAL_ERROR);
    }

    pub(crate) fn id(&self) -> &str
    {
        &self.id
    }

    pub(crate) fn info(&self) -> EntityInfo
    {
        use std::iter::FromIterator;

        let joined_environments = {
            let joined_environments = unlock!(self.joined_environments);
            HashSet::from_iter(joined_environments.keys().cloned())
        };

        let affecting_environments = {
            let affected_environments = unlock!(self.affected_environments);
            HashSet::from_iter(affected_environments.keys().cloned())
        };

        EntityInfo {
            num_received_effects: self.num_received_effects.load(Ordering::Relaxed),
            joined_environments,
            affecting_environments
        }
    }
}

impl Future for EntityHarness
{
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), Self::Error>
    {
        self.waker.task.register();

        {
            let num_effects = self.num_received_effects.load(Ordering::Acquire);
            let mut num = 0;

            let mut joined = unlock!(self.joined_environments);
            let affected = unlock!(self.affected_environments);
            let mut entity = unlock!(self.entity);

            let mut out_chan = unlock!(self.out_chan);
            let mut to_drop = vec![];

            'outer: loop {
                // number of dry in-channels
                let mut num_dry = 0;

                // Check each joined environment if there is a new effect
                for (
                    env_name,
                    Notifier {
                        effect_rx: env_rx,
                        drop_rx: _
                    }
                ) in joined.iter_mut()
                {
                    // Try to receive as many effects as possible from that
                    // environment TODO: maybe make this a
                    // for-loop with an upper limit to give other
                    // futures time to progress as well
                    'inner: loop {
                        match env_rx.try_recv() {
                            Ok(effect) => {
                                num += 1;

                                #[cfg(debug_assertions)]
                                println!(
                                    "entity \"{}\" received effect \"{:?}\" from environment \
                                     \"{}\" ({})",
                                    &self.id[0..5],
                                    effect,
                                    env_name,
                                    num_effects + num,
                                );

                                let context = Context { origin: &env_name };
                                let effect = entity.process(effect, context);

                                out_chan.broadcast(effect);

                                // TODO: rework this
                                // Wake all affected environments if half of the
                                // broadcaster buffer size is full
                                if num == BROADCAST_BUFFER_SIZE / 2 {
                                    for (_, env_waker) in affected.iter() {
                                        env_waker.task.notify();
                                    }
                                }
                            },
                            _ => {
                                num_dry += 1;
                                break 'inner;
                            }
                        }
                    }
                }

                // If all channels are dry this future can finally go to sleep
                // until awakened again
                if num_dry >= joined.len() {
                    break 'outer;
                }
            }

            self.num_received_effects
                .store(num_effects + num, Ordering::Release);

            // Wake all affected environments to process the remaining effects buffered in
            // the broadcast channel
            for (_, env_waker) in affected.iter() {
                env_waker.task.notify();
            }

            // Check if any environment is about to getting dropped by the supervisor
            for (
                env_name,
                Notifier {
                    effect_rx: _,
                    drop_rx: env_drop_rx
                }
            ) in joined.iter_mut()
            {
                match env_drop_rx.0.poll() {
                    Ok(Async::Ready(Some(is_drop_signal))) => {
                        if is_drop_signal {
                            #[cfg(debug_assertions)]
                            println!(
                                "entity \"{}\" received drop signal from environment \"{}\"",
                                &self.id[0..5],
                                env_name
                            );

                            to_drop.push(env_name.clone());
                        }
                    },
                    _ => ()
                }
            }

            // Remove all environments we received a drop signal from
            for env in to_drop {
                joined.remove(&env);

                #[cfg(debug_assertions)]
                println!(
                    "entity \"{}\" unsubscribed from environment '{}'",
                    &self.id[0..5],
                    env
                );
            }
        }

        // Check for system-shutdown event
        match unlock!(self.shutdown_listener).0.poll() {
            Ok(Async::Ready(Some(is_shutdown_signal))) => {
                if is_shutdown_signal {
                    #[cfg(debug_assertions)]
                    println!("entity \"{}\" received shutdown signal", &self.id[0..5]);

                    return Ok(Async::Ready(()));
                }
            },
            _ => ()
        }

        // Check for entity-termination event
        match unlock!(self.terminate_chan).1.poll() {
            Ok(Async::Ready(Some(is_term_signal))) => {
                if is_term_signal {
                    #[cfg(debug_assertions)]
                    println!("entity \"{}\" received termination signal", &self.id[0..5]);

                    return Ok(Async::Ready(()));
                }
            },
            _ => ()
        }

        Ok(Async::NotReady)
    }
}

impl Clone for EntityHarness
{
    fn clone(&self) -> Self
    {
        Self {
            id: self.id.clone(),
            joined_environments: Arc::clone(&self.joined_environments),
            affected_environments: Arc::clone(&self.affected_environments),
            out_chan: Arc::clone(&self.out_chan),
            drop_notifier: Arc::clone(&self.drop_notifier),
            shutdown_listener: Arc::clone(&self.shutdown_listener),
            terminate_chan: Arc::clone(&self.terminate_chan),
            waker: self.waker.clone(),
            num_received_effects: Arc::clone(&self.num_received_effects),
            entity: Arc::clone(&self.entity)
        }
    }
}

impl Drop for EntityHarness
{
    fn drop(&mut self)
    {
        #[cfg(debug_assertions)]
        println!("dropping entity \"{}\"", self.id());
    }
}
