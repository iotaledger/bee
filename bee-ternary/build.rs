// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

fn main() {
    let ac = autocfg::new();
    ac.emit_has_type("i128");
    ac.emit_has_type("u128");

    autocfg::rerun_path("build.rs");
}
