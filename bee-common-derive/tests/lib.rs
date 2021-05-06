// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[test]
fn tests() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/packable/newtype.rs");
    cases.pass("tests/packable/generic_newtype.rs");
    cases.pass("tests/packable/named_struct.rs");
    cases.pass("tests/packable/unnamed_struct.rs");
    cases.pass("tests/packable/named_enum.rs");
    cases.pass("tests/packable/unnamed_enum.rs");
    cases.pass("tests/packable/mixed_enum.rs");
    cases.compile_fail("tests/packable/no_ty_enum.rs");
    cases.compile_fail("tests/packable/invalid_ty_enum.rs");
    cases.compile_fail("tests/packable/no_id_enum.rs");
    cases.compile_fail("tests/packable/dup_id_enum.rs");
    cases.compile_fail("tests/packable/invalid_id_enum.rs");
    cases.compile_fail("tests/packable/packable_is_structural.rs");
    cases.compile_fail("tests/packable/union.rs");
}
