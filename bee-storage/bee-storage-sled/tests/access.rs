// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
#[allow(unused_macros)]
macro_rules! impl_access_test {
    ($name_sled:ident, $name:ident) => {
        #[test]
        fn $name_sled() {
            use bee_storage::backend::StorageBackend;

            let path = String::from("./tests/database/") + stringify!($name);
            let _ = std::fs::remove_dir_all(&path);

            let config = bee_storage_sled::config::SledConfigBuilder::default()
                .with_path(path.clone())
                .finish();
            let storage = bee_storage_sled::storage::Storage::start(config).unwrap();

            bee_storage_test::$name(&storage);

            let _ = std::fs::remove_dir_all(&path);
        }
    };
}
