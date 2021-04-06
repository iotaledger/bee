// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::password;

#[test]
fn correct_password() {
    let salt = password::generate_salt();
    let password_hash = password::password_hash(b"password", &salt).unwrap();

    assert!(password::password_verify(b"password", &salt, &password_hash).unwrap());
}

#[test]
fn incorrect_password() {
    let salt = password::generate_salt();
    let password_hash = password::password_hash(b"password", &salt).unwrap();

    assert!(!password::password_verify(b"passw0rd", &salt, &password_hash).unwrap());
}
