// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_storage::{
    access::{Fetch, Insert, MultiFetch},
    backend,
    system::{StorageHealth, StorageVersion, System, SYSTEM_HEALTH_KEY, SYSTEM_VERSION_KEY},
};

pub trait StorageBackend:
    backend::StorageBackend + Fetch<u8, System> + for<'a> MultiFetch<'a, u8, System> + Insert<u8, System>
// + for<'a> AsIterator<'a, u8, System>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend + Fetch<u8, System> + for<'a> MultiFetch<'a, u8, System> + Insert<u8, System> // + for<'a> AsIterator<'a, u8, System>
{
}

pub fn system_access<B: StorageBackend>(storage: &B) {
    let version = Fetch::<u8, System>::fetch(storage, &SYSTEM_VERSION_KEY)
        .unwrap()
        .unwrap();
    assert_eq!(version, System::Version(storage.version().unwrap().unwrap()));

    let health = Fetch::<u8, System>::fetch(storage, &SYSTEM_HEALTH_KEY)
        .unwrap()
        .unwrap();
    assert_eq!(health, System::Health(storage.health().unwrap().unwrap()));
    assert_eq!(health, System::Health(StorageHealth::Idle));

    assert_eq!(Fetch::<u8, System>::fetch(storage, &42).unwrap(), None);

    let systems = MultiFetch::<u8, System>::multi_fetch(storage, &[SYSTEM_VERSION_KEY, SYSTEM_HEALTH_KEY, 42])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(systems[0].as_ref().unwrap().unwrap(), version);
    assert_eq!(systems[1].as_ref().unwrap().unwrap(), health);
    assert_eq!(systems[2].as_ref().unwrap(), &None);

    Insert::<u8, System>::insert(storage, &SYSTEM_VERSION_KEY, &System::Version(StorageVersion(42))).unwrap();
    // assert_eq!(
    //     System::Version(storage.version().unwrap().unwrap()),
    //     System::Version(StorageVersion(42))
    // );
    assert_eq!(
        Fetch::<u8, System>::fetch(storage, &SYSTEM_VERSION_KEY)
            .unwrap()
            .unwrap(),
        System::Version(StorageVersion(42))
    );

    Insert::<u8, System>::insert(storage, &SYSTEM_HEALTH_KEY, &System::Health(StorageHealth::Corrupted)).unwrap();
    assert_eq!(
        System::Health(storage.health().unwrap().unwrap()),
        System::Health(StorageHealth::Corrupted)
    );
    assert_eq!(
        Fetch::<u8, System>::fetch(storage, &SYSTEM_HEALTH_KEY)
            .unwrap()
            .unwrap(),
        System::Health(StorageHealth::Corrupted)
    );

    // let iter = AsIterator::<u8, System>::iter(storage).unwrap();
    // let mut count = 0;

    // for result in iter {
    //     let (message_id, message) = result.unwrap();
    //     assert!(messages.contains(&(message_id, Some(message))));
    //     count += 1;
    // }
    //
    // assert_eq!(count, 10);
    //
}
