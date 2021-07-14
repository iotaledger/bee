// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::Packable;

use core::convert::TryInto;

const BEACON_PARTIAL_PUBLIC_KEY: &str = "55914b063d6342d89680c90b3617877c0dd5c1b88fce7e19d24904ebe56aaca9835d458d77f61\
    bb2a250805e25ab6be095f2a498419f89056157b29cb088271c93253e1b420f52d893abe4d76be718964d0f322991a253ef6a66c17ec586244\
    1";

const BEACON_SIGNATURE: &str = "19d74b5699134c94c0188f72fbd76b6463e2f52c3f0c126b8cefc87502234e62e7202996b52dea13318bec\
    0b451ac67a1346f803e4827900610698c7d48f426c4bf459a3172ce2a5107ef58e90a7d24542c517b3201371bea9f4a04d8a0ab0cc";

#[test]
fn kind() {
    assert_eq!(BeaconPayload::KIND, 5);
}

#[test]
fn new_valid() {
    let beacon = BeaconPayload::builder()
        .with_version(0)
        .with_instance_id(0)
        .with_round(1)
        .with_partial_public_key(hex::decode(BEACON_PARTIAL_PUBLIC_KEY).unwrap().try_into().unwrap())
        .with_partial_signature(hex::decode(BEACON_SIGNATURE).unwrap().try_into().unwrap())
        .finish();

    assert!(beacon.is_ok());
}

#[test]
fn unpack_valid() {
    let mut bytes = vec![0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];

    bytes.extend(hex::decode(BEACON_PARTIAL_PUBLIC_KEY).unwrap());
    bytes.extend(hex::decode(BEACON_SIGNATURE).unwrap());

    assert!(BeaconPayload::unpack_from_slice(bytes).is_ok());
}

#[test]
fn accessors_eq() {
    let partial_pk: [u8; 96] = hex::decode(BEACON_PARTIAL_PUBLIC_KEY).unwrap().try_into().unwrap();
    let partial_signature: [u8; 96] = hex::decode(BEACON_SIGNATURE).unwrap().try_into().unwrap();

    let beacon = BeaconPayload::builder()
        .with_version(0)
        .with_instance_id(0)
        .with_round(1)
        .with_partial_public_key(partial_pk)
        .with_partial_signature(partial_signature)
        .finish()
        .unwrap();

    assert_eq!(beacon.version(), 0);
    assert_eq!(beacon.instance_id(), 0);
    assert_eq!(beacon.round(), 1);
    assert_eq!(*beacon.partial_public_key(), partial_pk);
    assert_eq!(*beacon.partial_signature(), partial_signature);
}

#[test]
fn packed_len() {
    let beacon = BeaconPayload::builder()
        .with_version(0)
        .with_instance_id(0)
        .with_round(1)
        .with_partial_public_key(hex::decode(BEACON_PARTIAL_PUBLIC_KEY).unwrap().try_into().unwrap())
        .with_partial_signature(hex::decode(BEACON_SIGNATURE).unwrap().try_into().unwrap())
        .finish()
        .unwrap();

    assert_eq!(beacon.packed_len(), 1 + 4 + 8 + 96 + 96);
}

#[test]
fn round_trip() {
    let beacon_a = BeaconPayload::builder()
        .with_version(0)
        .with_instance_id(0)
        .with_round(1)
        .with_partial_public_key(hex::decode(BEACON_PARTIAL_PUBLIC_KEY).unwrap().try_into().unwrap())
        .with_partial_signature(hex::decode(BEACON_SIGNATURE).unwrap().try_into().unwrap())
        .finish()
        .unwrap();

    let beacon_b = BeaconPayload::unpack_from_slice(beacon_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(beacon_a, beacon_b);
}
