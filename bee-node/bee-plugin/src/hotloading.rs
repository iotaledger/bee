// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Hotloading related utilities.

use crate::{PluginError, PluginId, PluginManager, UniqueId};

use bee_event_bus::EventBus;
use tokio::{
    process::Command,
    time::{sleep, Duration},
};

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

const PLUGIN_CHECK_INTERVAL_MILLIS: u64 = 1000;

/// Type that watches a directory for changes and loads every file as a plugin.
pub struct Hotloader {
    directory: PathBuf,
    plugins_info: HashMap<PathBuf, PluginInfo>,
    manager: PluginManager,
}

impl Hotloader {
    /// Creates a new [`Hotloader`] that watches the specified directory.
    pub fn new<P: AsRef<Path> + ?Sized>(directory: &P, event_bus: Arc<EventBus<'static, UniqueId>>) -> Self {
        Self {
            directory: directory.as_ref().to_owned(),
            plugins_info: HashMap::new(),
            manager: PluginManager::new(event_bus),
        }
    }

    /// Watches for changes in a directory and loads new plugins.
    ///
    /// This is done by following these rules:
    /// - If a file is created, it will be loaded and executed as a plugin.
    /// - If a file is removed, the process for it will be terminated and the plugin will be
    /// unloaded.
    /// - If a file is modified, it will behave as if the file was removed and created in
    /// succession.
    pub async fn run(mut self) -> Result<(), PluginError> {
        loop {
            let mut dir = tokio::fs::read_dir(&self.directory).await?;

            let mut last_writes = HashMap::new();

            while let Some(entry) = dir.next_entry().await? {
                let path = entry.path();
                let metadata = entry.metadata().await?;

                if metadata.is_file() {
                    last_writes.insert(path, metadata.modified()?);
                }
            }

            let mut to_delete = Vec::new();

            for (path, info) in self.plugins_info.iter_mut() {
                match last_writes.remove(path) {
                    // The file still exists
                    Some(last_write) => {
                        if info.last_write != last_write {
                            // The file changed
                            self.manager.unload(info.id).await?;
                            let command = Command::new(path);
                            info.id = self.manager.load(command).await?;
                            info.last_write = last_write;
                        }
                    }
                    // The file no longer exists
                    None => {
                        to_delete.push(path.clone());
                        self.manager.unload(info.id).await?;
                    }
                }
            }

            // remove info for files that no longer exist and load plugins whose files did not
            // exist before.
            self.sync_plugin_info(to_delete, last_writes).await?;

            sleep(Duration::from_millis(PLUGIN_CHECK_INTERVAL_MILLIS)).await;
        }
    }

    async fn sync_plugin_info(
        &mut self,
        to_delete: Vec<PathBuf>,
        to_load: HashMap<PathBuf, SystemTime>,
    ) -> Result<(), PluginError> {
        for path in to_delete {
            self.plugins_info.remove(&path);
        }
        for (path, last_write) in to_load {
            let command = Command::new(&path);
            let id = self.manager.load(command).await?;
            self.plugins_info.insert(path, PluginInfo { id, last_write });
        }

        Ok(())
    }
}

struct PluginInfo {
    id: PluginId,
    last_write: SystemTime,
}
