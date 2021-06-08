// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use alloc::vec::Vec;

#[test]
fn is_sorted() {
    let vec = Vec::from([0, 1, 2, 3, 5, 7, 10, 12, 29]);
    assert!(bee_ord::is_sorted(vec.iter()));
}

#[test]
fn not_sorted() {
    let vec = Vec::from([0, 1, 2, 3, 5, 7, 10, 29, 12]);
    assert!(!bee_ord::is_sorted(vec.iter()));
}

#[test]
fn is_unique_sorted() {
    let vec = Vec::from([0, 1, 2, 3, 5, 7, 10, 12, 29]);
    assert!(bee_ord::is_unique_sorted(vec.iter()));
}

#[test]
fn sorted_not_unique() {
    let vec = Vec::from([0, 1, 2, 3, 5, 7, 10, 12, 12]);
    assert!(bee_ord::is_sorted(vec.iter()));
    assert!(!bee_ord::is_unique_sorted(vec.iter()));
}

#[test]
fn not_sorted_not_unique() {
    let vec = Vec::from([0, 1, 2, 3, 5, 7, 12, 10, 12]);
    assert!(!bee_ord::is_sorted(vec.iter()));
    assert!(!bee_ord::is_unique_sorted(vec.iter()));
}
