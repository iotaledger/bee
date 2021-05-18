// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use trybuild::TestCases;

use std::path::Path;

macro_rules! make_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let path = Path::new("tests/").join(stringify!($name));
            let cases = TestCases::new();

            cases.pass(path.join("pass/*.rs"));
            cases.compile_fail(path.join("fail/*.rs"));
        }
    };
}

make_test!(packable);
