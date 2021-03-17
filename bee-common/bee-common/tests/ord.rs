// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::ord;

#[test]
fn is_unique_sorted_empty() {
    assert!(ord::is_unique_sorted(Vec::<()>::new().iter()));
}

#[test]
fn is_unique_sorted() {
    assert!(ord::is_unique_sorted(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9].iter()));
}

#[test]
fn is_unique_not_sorted() {
    assert!(!ord::is_unique_sorted(vec![0, 1, 3, 2, 4, 5, 6, 7, 8, 9].iter()));
}

#[test]
fn is_not_unique_sorted() {
    assert!(!ord::is_unique_sorted(vec![0, 1, 1, 2, 3, 4, 5, 6, 7, 8].iter()));
}

#[test]
fn is_sorted_empty() {
    assert!(ord::is_sorted(Vec::<()>::new().iter()));
}

#[test]
fn is_sorted() {
    assert!(ord::is_sorted(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9].iter()));
}

#[test]
fn is_not_sorted() {
    assert!(!ord::is_sorted(vec![0, 1, 3, 2, 4, 5, 6, 7, 8, 9].iter()));
}

#[test]
fn is_sorted_not_unique() {
    assert!(ord::is_sorted(vec![0, 1, 1, 2, 3, 4, 5, 6, 7, 8, 9].iter()));
}

#[test]
fn vec_is_sorted() {
    assert!(ord::is_sorted(
        vec![
            vec![0, 1, 'a' as u8, 2, 3, 4, 5, 6, 7, 8, 9],
            vec![0, 1, 'b' as u8, 2, 3, 4, 5, 6, 7, 8, 9],
        ]
        .iter()
    ));
}

#[test]
fn vec_is_not_sorted() {
    assert!(!ord::is_sorted(
        vec![
            vec![0, 1, 'b' as u8, 2, 3, 4, 5, 6, 7, 8, 9],
            vec![0, 1, 'a' as u8, 2, 3, 4, 5, 6, 7, 8, 9],
        ]
        .iter()
    ));
}
