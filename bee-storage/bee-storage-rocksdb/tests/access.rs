// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_macros)]
macro_rules! impl_access_test {
    ($name_rocksdb:ident, $name:ident) => {
        #[test]
        fn $name_rocksdb() {
            use bee_storage::backend::StorageBackend;

            let path = String::from("./tests/database/") + stringify!($name);
            let _ = std::fs::remove_dir_all(&path);

            let config = bee_storage_rocksdb::config::RocksDbConfigBuilder::default()
                .with_path((&path).into())
                .finish();
            let storage = bee_storage_rocksdb::storage::Storage::start(config).unwrap();

            bee_storage_test::$name(&storage);

            let _ = std::fs::remove_dir_all(&path);
        }
    };
}
