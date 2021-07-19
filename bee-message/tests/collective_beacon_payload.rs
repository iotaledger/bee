// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::Packable;

use core::convert::TryInto;

const BEACON_SIGNATURE_0: &str = "55914b063d6342d89680c90b3617877c0dd5c1b88fce7e19d24904ebe56aaca9835d458d77f61bb2a250\
    805e25ab6be095f2a498419f89056157b29cb088271c93253e1b420f52d893abe4d76be718964d0f322991a253ef6a66c17ec5862441";

const BEACON_SIGNATURE_1: &str = "19d74b5699134c94c0188f72fbd76b6463e2f52c3f0c126b8cefc87502234e62e7202996b52dea13318b\
    ec0b451ac67a1346f803e4827900610698c7d48f426c4bf459a3172ce2a5107ef58e90a7d24542c517b3201371bea9f4a04d8a0ab0cc";

const BEACON_DISTRIBUTED_PUBLIC_KEY: &str = "400c0eb2d345379af16e8cfc567a89917eb2c2e3caa49358a6c6bc1386a1b4abe0a6c5ac3\
    f93808df4fee169b61f20a5";

#[test]
fn kind() {
    assert_eq!(CollectiveBeaconPayload::KIND, 6);
}

#[test]
fn new_valid() {
    let beacon = CollectiveBeaconPayload::builder()
        .with_version(0)
        .with_instance_id(0)
        .with_round(1)
        .with_prev_signature(hex::decode(BEACON_SIGNATURE_0).unwrap().try_into().unwrap())
        .with_signature(hex::decode(BEACON_SIGNATURE_1).unwrap().try_into().unwrap())
        .with_distributed_public_key(hex::decode(BEACON_DISTRIBUTED_PUBLIC_KEY).unwrap().try_into().unwrap())
        .finish();

    assert!(beacon.is_ok());
}

#[test]
fn unpack_valid() {
    let mut bytes = vec![0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];

    bytes.extend(hex::decode(BEACON_SIGNATURE_0).unwrap());
    bytes.extend(hex::decode(BEACON_SIGNATURE_1).unwrap());
    bytes.extend(hex::decode(BEACON_DISTRIBUTED_PUBLIC_KEY).unwrap());

    assert!(CollectiveBeaconPayload::unpack_from_slice(bytes).is_ok());
}

#[test]
fn accessors_eq() {
    let prev_signature: [u8; 96] = hex::decode(BEACON_SIGNATURE_0).unwrap().try_into().unwrap();
    let signature: [u8; 96] = hex::decode(BEACON_SIGNATURE_1).unwrap().try_into().unwrap();
    let distributed_pk: [u8; 48] = hex::decode(BEACON_DISTRIBUTED_PUBLIC_KEY).unwrap().try_into().unwrap();

    let beacon = CollectiveBeaconPayload::builder()
        .with_version(0)
        .with_instance_id(0)
        .with_round(1)
        .with_prev_signature(prev_signature)
        .with_signature(signature)
        .with_distributed_public_key(distributed_pk)
        .finish()
        .unwrap();

    assert_eq!(beacon.version(), 0);
    assert_eq!(beacon.instance_id(), 0);
    assert_eq!(beacon.round(), 1);
    assert_eq!(beacon.prev_signature(), prev_signature);
    assert_eq!(beacon.signature(), signature);
    assert_eq!(beacon.distributed_public_key(), distributed_pk);
}

#[test]
fn packed_len() {
    let beacon = CollectiveBeaconPayload::builder()
        .with_version(0)
        .with_instance_id(0)
        .with_round(1)
        .with_prev_signature(hex::decode(BEACON_SIGNATURE_0).unwrap().try_into().unwrap())
        .with_signature(hex::decode(BEACON_SIGNATURE_1).unwrap().try_into().unwrap())
        .with_distributed_public_key(hex::decode(BEACON_DISTRIBUTED_PUBLIC_KEY).unwrap().try_into().unwrap())
        .finish()
        .unwrap();

    assert_eq!(beacon.packed_len(), 1 + 4 + 8 + 96 + 96 + 48);
}

#[test]
fn packable_round_trip() {
    let beacon_a = CollectiveBeaconPayload::builder()
        .with_version(0)
        .with_instance_id(0)
        .with_round(1)
        .with_prev_signature(hex::decode(BEACON_SIGNATURE_0).unwrap().try_into().unwrap())
        .with_signature(hex::decode(BEACON_SIGNATURE_1).unwrap().try_into().unwrap())
        .with_distributed_public_key(hex::decode(BEACON_DISTRIBUTED_PUBLIC_KEY).unwrap().try_into().unwrap())
        .finish()
        .unwrap();

    let beacon_b = CollectiveBeaconPayload::unpack_from_slice(beacon_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(beacon_a, beacon_b);
}
