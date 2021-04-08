// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "password")]
use bee_common::password;

#[test]
#[cfg(feature = "password")]
fn correct_password() {
    let salt = password::generate_salt();
    let password_hash = password::password_hash(b"password", &salt).unwrap();

    assert!(password::password_verify(b"password", &salt, &password_hash).unwrap());
}

#[test]
#[cfg(feature = "password")]
fn incorrect_password() {
    let salt = password::generate_salt();
    let password_hash = password::password_hash(b"password", &salt).unwrap();

    assert!(!password::password_verify(b"passw0rd", &salt, &password_hash).unwrap());
}
