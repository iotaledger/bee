// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_macros)]
macro_rules! impl_access_test {
    ($name_memory:ident, $name:ident) => {
        #[test]
        fn $name_memory() {
            use bee_storage::backend::StorageBackend;

            let storage = bee_storage_memory::storage::Storage::start(()).unwrap();

            bee_storage_test::$name(&storage);
        }
    };
}
