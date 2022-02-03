// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use trybuild::TestCases;

use std::{ffi::OsStr, fs::read_dir, path::Path};

fn for_each_entry(root_path: &Path, f: impl Fn(&Path)) {
    let rs_ext = Some(OsStr::new("rs"));
    for entry in read_dir(root_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension() == rs_ext {
            f(&path);
        }
    }
}

macro_rules! make_failing_tests {
    () => {
        #[test]
        fn packable() {
            let path = Path::new("tests/");
            let cases = TestCases::new();

            for_each_entry(&path.join("fail/"), |path| {
                cases.compile_fail(path);
            });
        }
    };
}

make_failing_tests!();
