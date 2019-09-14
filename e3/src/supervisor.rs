use crate::common::signal::SignalRx;
use crate::common::waker::Waker;
use crate::constants::messages::*;
use crate::effect::Effect;
use crate::entity::{Entity, EntityHarness, EntityInfo};
use crate::environment::{Environment, EnvironmentInfo};

use std::collections::HashMap as Map;
use std::sync::{Arc, Mutex};

use crossbeam_channel::{unbounded, Sender};
use tokio::prelude::*;

use bus::BusReader as BroadcastReceiver;

pub(crate) struct Supervisor
{
    inner: Arc<Mutex<SupervisorInner>>
}

struct SupervisorInner
{
    environments: Map<String, EnvironmentConnection>,
    entities: Map<String, EntityConnection>,
    shutdown_rx: SignalRx
}

struct EnvironmentConnection
{
    pub(crate) environment: Environment,
    pub(crate) effect_tx: Sender<Effect>,
    pub(crate) waker: Waker
}

struct EntityConnection
{
    pub(crate) entity: EntityHarness
}

pub(crate) struct Notifier
{
    pub(crate) effect_rx: BroadcastReceiver<Effect>,
    pub(crate) drop_rx: SignalRx
}

impl Supervisor
{
    pub(crate) fn new(shutdown_rx: SignalRx) -> Self
    {
        let inner = Arc::new(Mutex::new(SupervisorInner {
            environments: Map::new(),
            entities: Map::new(),
            shutdown_rx
        }));

        Self { inner }
    }

    pub(crate) fn create_environment(&mut self, name: &str, term_rx: SignalRx) -> Environment
    {
        let mut sv = unlock_msg!(self.inner, UNLOCK_SUPERVISOR_ERROR);

        if sv.environments.contains_key(name) {
            panic!(ENVIRONMENT_ALREADY_EXISTS_ERROR);
        }

        let (effect_tx, effect_rx) = unbounded();

        let env = Environment::new(name, effect_rx, term_rx);

        let conn = EnvironmentConnection {
            environment: env.clone(),
            effect_tx,
            waker: env.get_waker()
        };

        sv.environments.insert(name.into(), conn);

        #[cfg(debug_assertions)]
        println!("supervisor created environment \"{}\"", name);

        env
    }

    pub(crate) fn delete_environment(&mut self, name: &str)
    {
        let mut sv = unlock_msg!(self.inner, UNLOCK_SUPERVISOR_ERROR);

        match sv.environments.remove(name) {
            Some(env_conn) => {
                env_conn.environment.terminate();
            },
            None => panic!(ENVIRONMENT_DOESNT_EXIST_ERROR)
        }

        #[cfg(debug_assertions)]
        println!("supervisor deleted environment \"{}\"", name);
    }

    pub(crate) fn environment_exists(&self, name: &str) -> bool
    {
        unlock_msg!(self.inner, UNLOCK_SUPERVISOR_ERROR)
            .environments
            .contains_key(name)
    }

    pub(crate) fn register_entity(
        &mut self,
        entity: impl Entity + 'static,
        shutdown_rx: SignalRx
    ) -> EntityHarness
    {
        let entity = EntityHarness::new(Box::new(entity), shutdown_rx);

        let mut sv = unlock_msg!(self.inner, UNLOCK_SUPERVISOR_ERROR);
        sv.entities.insert(
            entity.id().into(),
            EntityConnection {
                entity: entity.clone()
            }
        );

        #[cfg(debug_assertions)]
        println!("supervisor registered entity \"{}\"", &entity.id()[..5]);

        entity
    }

    pub(crate) fn deregister_entity(&mut self, id: &str)
    {
        let mut sv = unlock!(self.inner);

        match sv.entities.remove(id) {
            Some(ent_conn) => {
                ent_conn.entity.terminate();
            },
            None => panic!(ENTITY_DOESNT_EXIST_ERROR)
        }

        #[cfg(debug_assertions)]
        println!("supervisor deregistered entity \"{}\"", &id[..5]);
    }

    pub(crate) fn entity_exists(&self, id: &str) -> bool
    {
        unlock_msg!(self.inner, UNLOCK_SUPERVISOR_ERROR)
            .entities
            .contains_key(id)
    }

    pub(crate) fn join_environment(&mut self, entity_id: &str, environment_name: &str) -> bool
    {
        let mut sv = unlock!(self.inner);

        if !sv.environments.contains_key(environment_name) || !sv.entities.contains_key(entity_id) {
            return false;
        }

        let (env_effect_rx, env_drop_rx) = {
            let env_link = sv.environments.get_mut(environment_name).unwrap();
            env_link.environment.get_connectors()
        };

        let joined_entity = {
            let ent_link = sv.entities.get_mut(entity_id).unwrap();
            ent_link
                .entity
                .connect_input(&environment_name, env_effect_rx, env_drop_rx)
        };

        let env_link = sv.environments.get_mut(environment_name).unwrap();
        env_link
            .environment
            .connect_output(entity_id, joined_entity);

        #[cfg(debug_assertions)]
        println!(
            "entity \"{}\" joined environment \"{}\"",
            &entity_id[..5],
            environment_name
        );

        true
    }

    pub(crate) fn leave_environment(&mut self, entity_id: &str, environment_name: &str) -> bool
    {
        let mut sv = unlock!(self.inner);

        if !sv.environments.contains_key(environment_name) || !sv.entities.contains_key(entity_id) {
            return false;
        }

        {
            let ent_link = sv.entities.get_mut(entity_id).unwrap();
            ent_link.entity.disconnect_input(environment_name);
        }
        {
            let env_link = sv.environments.get_mut(environment_name).unwrap();
            env_link.environment.disconnect_output(entity_id);
        }

        #[cfg(debug_assertions)]
        println!(
            "entity \"{}\" left environment \"{}\"",
            &entity_id[..5],
            environment_name
        );

        true
    }

    pub(crate) fn affect_environment(&mut self, entity_id: &str, environment_name: &str) -> bool
    {
        let mut sv = unlock!(self.inner);

        if !sv.environments.contains_key(environment_name) || !sv.entities.contains_key(entity_id) {
            return false;
        }

        let (ent_effect_rx, ent_drop_rx) = {
            let ent_link = sv.entities.get_mut(entity_id).unwrap();
            ent_link.entity.get_connectors()
        };

        let env_waker = {
            let env_link = sv.environments.get_mut(environment_name).unwrap();
            env_link
                .environment
                .connect_input(&entity_id, ent_effect_rx, ent_drop_rx)
        };

        let ent_link = sv.entities.get_mut(entity_id).unwrap();
        ent_link.entity.connect_output(environment_name, env_waker);

        #[cfg(debug_assertions)]
        println!(
            "entity \"{}\" now affects environment \"{}\"",
            &entity_id[..5],
            environment_name
        );

        true
    }

    pub(crate) fn ignore_environment(&mut self, entity_id: &str, environment_name: &str) -> bool
    {
        let mut sv = unlock!(self.inner);

        if !sv.environments.contains_key(environment_name) || !sv.entities.contains_key(entity_id) {
            return false;
        }

        {
            let env_link = sv.environments.get_mut(environment_name).unwrap();
            env_link.environment.disconnect_input(entity_id);
        }

        {
            let ent_link = sv.entities.get_mut(entity_id).unwrap();
            ent_link.entity.disconnect_output(environment_name);
        }

        #[cfg(debug_assertions)]
        println!(
            "entity \"{}\" now ignores environment \"{}\"",
            &entity_id[..5],
            environment_name
        );

        true
    }

    pub(crate) fn send_effect(&mut self, effect: Effect, environment_name: &str) -> bool
    {
        let sv = unlock!(self.inner);

        let result = match sv.environments.get(environment_name) {
            Some(env_link) => match env_link.effect_tx.send(effect) {
                Err(_) => false,
                _ => {
                    env_link.waker.task.notify();
                    true
                }
            },
            None => false
        };

        #[cfg(debug_assertions)]
        println!(
            "supervisor sent effect to environment \"{}\" (success={})",
            environment_name, result
        );

        result
    }

    pub(crate) fn num_environments(&self) -> usize
    {
        unlock!(self.inner).environments.len()
    }

    pub(crate) fn num_entities(&self) -> usize
    {
        unlock!(self.inner).entities.len()
    }

    pub(crate) fn get_environment_info(&self, environment_name: &str) -> Option<EnvironmentInfo>
    {
        let sv = unlock_msg!(self.inner, UNLOCK_SUPERVISOR_ERROR);

        if !sv.environments.contains_key(environment_name) {
            return None;
        }

        if let Some(conn) = sv.environments.get(environment_name) {
            return Some(conn.environment.info());
        };

        None
    }

    pub(crate) fn get_entity_info(&self, entity_id: &str) -> Option<EntityInfo>
    {
        let sv = unlock_msg!(self.inner, UNLOCK_SUPERVISOR_ERROR);

        if !sv.entities.contains_key(entity_id) {
            return None;
        }

        if let Some(conn) = sv.entities.get(entity_id) {
            return Some(conn.entity.info());
        };

        None
    }
}

impl Future for Supervisor
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), Self::Error>
    {
        let mut sv = unlock!(self.inner);

        match sv.shutdown_rx.0.poll() {
            Ok(Async::Ready(Some(is_shutdown_signal))) => {
                if is_shutdown_signal {
                    #[cfg(debug_assertions)]
                    println!("supervisor received shutdown signal");

                    return Ok(Async::Ready(()));
                }
            },
            _ => ()
        }

        return Ok(Async::NotReady);
    }
}

impl Clone for Supervisor
{
    fn clone(&self) -> Self
    {
        Self {
            inner: Arc::clone(&self.inner)
        }
    }
}

impl Drop for Supervisor
{
    fn drop(&mut self)
    {
        #[cfg(debug_assertions)]
        println!("dropping supervisor");
    }
}

#[cfg(test)]
mod should
{
    use super::*;
    use crate::common::signal::Signal;

    #[test]
    #[should_panic]
    fn fail_if_created_two_environments_with_the_same_name()
    {
        let signal = Signal::new();

        let mut sv = Supervisor::new(signal.add_rx());

        sv.create_environment("X", signal.add_rx());
        sv.create_environment("X", signal.add_rx());
    }

    #[test]
    fn count_two_for_two_different_environments()
    { /*
         let mut sv = Supervisor::new();

         sv.create_environment("X").unwrap();
         sv.create_environment("Y").unwrap();

         assert_eq!(2, sv.num_environments());
         */
    }

    // Cannot create the same environment twice
    // #[should_panic]
    // #[test]
    // fn forbid_creating_the_same_environment_twice() {
    // let mut sv = Supervisor::new().unwrap();
    //
    // sv.create_environment("X").unwrap();
    // sv.create_environment("X").unwrap();
    // }
    //
    // #[test]
    // fn create_and_delete_environment() {
    // let mut sv = Supervisor::new().unwrap();
    //
    // let x = sv.create_environment("X").unwrap();
    // assert_eq!(1, sv.num_environments());
    //
    // sv.delete_environment(&x.name()).unwrap();
    // assert_eq!(0, sv.num_environments());
    // }
    //
    // #[test]
    // fn submit_two_effects() {
    // let mut sv = Supervisor::new().unwrap();
    //
    // let x = sv.create_environment("X").unwrap();
    // let mut a = sv.create_entity().unwrap();
    //
    // sv.join_environments(&mut a, vec![&x.name()]).unwrap();
    //
    // sv.submit_effect("hello", &x.name()).unwrap();
    // sv.submit_effect("world", &x.name()).unwrap();
    //
    // Wait a little until the effects have propagated
    // std::thread::sleep(std::time::Duration::from_millis(100));
    //
    // assert_eq!(2, x.num_received_effects());
    // assert_eq!(2, a.num_received_effects());
    // }
    //
    // #[test]
    // fn submit_many_effects_to_two_entities() {
    // let mut sv = Supervisor::new().unwrap();
    //
    // let x = sv.create_environment("X").unwrap();
    //
    // let mut a = sv.create_entity().unwrap();
    // let mut b = sv.create_entity().unwrap();
    //
    // sv.join_environments(&mut a, vec![&x.name()]).unwrap();
    // sv.join_environments(&mut b, vec![&x.name()]).unwrap();
    //
    // for i in 0..729 {
    // sv.submit_effect(&i.to_string(), &x.name()).unwrap();
    // }
    //
    // Wait a little until the effects have propagated
    // std::thread::sleep(std::time::Duration::from_millis(100));
    //
    // assert_eq!(729, x.num_received_effects());
    // assert_eq!(729, a.num_received_effects());
    // assert_eq!(729, b.num_received_effects());
    // }
}
