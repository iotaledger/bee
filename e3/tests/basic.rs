use e3::prelude::*;

use std::thread;
use std::time::Duration;

struct MyEntity;
impl Entity for MyEntity {
    fn process(&mut self, _: Effect, _: Context) -> Effect {
        Effect::Empty
    }
}

fn get_num_received_effects_at_environment(env_name: &str) -> usize {
    e3::get_environment_info(env_name)
        .unwrap()
        .num_received_effects()
}

fn get_num_received_effects_at_entity(ent_id: &str) -> usize {
    e3::get_entity_info(ent_id).unwrap().num_received_effects()
}

#[test]
fn entity_receiving_effect_from_joined_environment() {
    e3::init();

    let x = &e3::create_environment("X");
    let a = &e3::register_entity(MyEntity);

    e3::join_environment(a, x);

    e3::send_effect(Effect::from("hello"), x);

    thread::sleep(Duration::from_millis(500));

    assert_eq!(1, get_num_received_effects_at_environment(x));
    assert_eq!(1, get_num_received_effects_at_entity(a));

    e3::shutdown();
}

#[test]
fn entities_receiving_effects_from_multiple_joined_environments() {
    e3::init();

    let x = &e3::create_environment("X");
    let y = &e3::create_environment("Y");

    let a = &e3::register_entity(MyEntity);
    let b = &e3::register_entity(MyEntity);

    e3::join_environment(a, x);
    e3::join_environment(a, y);
    e3::join_environment(b, y);

    e3::send_effect(Effect::from("hello"), x);
    e3::send_effect(Effect::from("world"), y);

    thread::sleep(Duration::from_millis(500));

    assert_eq!(1, get_num_received_effects_at_environment(x));
    assert_eq!(1, get_num_received_effects_at_environment(y));
    assert_eq!(2, get_num_received_effects_at_entity(a));
    assert_eq!(1, get_num_received_effects_at_entity(b));

    e3::shutdown();
}

#[test]
fn sending_multiple_effects() {
    e3::init();

    let x = &e3::create_environment("X");
    let a = &e3::register_entity(MyEntity);

    e3::join_environment(a, x);

    for c in "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890".chars() {
        e3::send_effect(Effect::from(c), x);
    }

    thread::sleep(Duration::from_millis(500));

    assert_eq!(36, get_num_received_effects_at_environment(x));
    assert_eq!(36, get_num_received_effects_at_entity(a));

    e3::shutdown();
}

#[test]
fn entity_joining_and_affecting() {
    e3::init();

    let x = &e3::create_environment("X");
    let y = &e3::create_environment("Y");

    let a = &e3::register_entity(MyEntity);

    e3::join_environment(a, x);
    e3::affect_environment(a, y);

    e3::send_effect(Effect::from("hello"), x);

    thread::sleep(Duration::from_millis(500));

    assert_eq!(1, get_num_received_effects_at_entity(a));
    assert_eq!(1, get_num_received_effects_at_environment(y));

    e3::shutdown();
}

#[test]
fn entity_joining_and_affecting_mutliple_environments() {
    e3::init();

    let x = &e3::create_environment("X");
    let y = &e3::create_environment("Y");
    let z = &e3::create_environment("Z");

    let a = &e3::register_entity(MyEntity);

    e3::join_environment(a, x);
    e3::affect_environment(a, y);
    e3::affect_environment(a, z);

    e3::send_effect(Effect::from("hello"), x);

    thread::sleep(Duration::from_millis(500));

    e3::shutdown();
}

struct StringReverse;
impl Entity for StringReverse {
    fn process(&mut self, effect: Effect, _: Context) -> Effect {
        let result = match effect {
            Effect::String(s) => Effect::from(s.chars().rev().collect::<String>()),
            _ => Effect::Empty,
        };
        result
    }
}

struct StringUppercase;
impl Entity for StringUppercase {
    fn process(&mut self, effect: Effect, _: Context) -> Effect {
        let result = match effect {
            Effect::String(s) => Effect::from(s.to_uppercase()),
            _ => Effect::Empty,
        };
        result
    }
}

#[test]
fn two_entities_manipulating_the_same_effect() {
    e3::init();

    let x = &e3::create_environment("X");
    let y = &e3::create_environment("Y");

    let a = &e3::register_entity(StringReverse);
    let b = &e3::register_entity(StringUppercase);

    e3::join_environment(a, x);
    e3::join_environment(b, x);

    e3::affect_environment(a, y);
    e3::affect_environment(b, y);

    e3::send_effect(Effect::from("hello"), x);

    thread::sleep(Duration::from_millis(500));

    // assert that one effect arrived as 'olleh'
    // assert that one effect arrived as 'HELLO'

    e3::shutdown();
}
