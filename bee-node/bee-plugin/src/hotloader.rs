// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Plugins hotloading utilities.

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

struct PluginInfo {
    id: PluginId,
    last_write: SystemTime,
}

/// Type that watches a directory for changes and loads every file as a plugin.
pub struct PluginHotloader {
    directory: PathBuf,
    plugins_info: HashMap<PathBuf, PluginInfo>,
    manager: PluginManager,
}

impl PluginHotloader {
    /// Creates a new [`PluginHotloader`] that watches the specified directory.
    pub fn new<P: AsRef<Path> + ?Sized>(directory: &P, bus: Arc<EventBus<'static, UniqueId>>) -> Self {
        Self {
            directory: directory.as_ref().to_owned(),
            plugins_info: HashMap::new(),
            manager: PluginManager::new(bus),
        }
    }

    /// Watches for changes in a directory and loads new plugins.
    ///
    /// This is done by following these rules:
    /// - If a file is created, it will be loaded and executed as a plugin.
    /// - If a file is removed, the process for it will be terminated and the plugin will be unloaded.
    /// - If a file is modified, it will behave as if the file was removed and created in succession.
    pub async fn run(mut self) -> Result<(), PluginError> {
        loop {
            let mut dir = tokio::fs::read_dir(&self.directory).await?;
            let mut last_writes = HashMap::new();
            let mut to_remove = Vec::new();

            while let Some(entry) = dir.next_entry().await? {
                let path = entry.path();
                let metadata = entry.metadata().await?;

                if metadata.is_file() {
                    last_writes.insert(path, metadata.modified()?);
                }
            }

            for (path, info) in self.plugins_info.iter_mut() {
                match last_writes.remove(path) {
                    // The file still exists.
                    Some(last_write) => {
                        if info.last_write != last_write {
                            // The file changed.
                            self.manager.unload(info.id).await?;
                            let command = Command::new(path);
                            info.id = self.manager.load(command).await?;
                            info.last_write = last_write;
                        }
                    }
                    // The file no longer exists.
                    None => {
                        to_remove.push(path.clone());
                        self.manager.unload(info.id).await?;
                    }
                }
            }

            // Removes info for files that no longer exist and loads plugins whose files did not exist before.
            self.sync_plugin_info(to_remove, last_writes).await?;

            sleep(Duration::from_millis(PLUGIN_CHECK_INTERVAL_MILLIS)).await;
        }
    }

    async fn sync_plugin_info(
        &mut self,
        to_remove: Vec<PathBuf>,
        to_load: HashMap<PathBuf, SystemTime>,
    ) -> Result<(), PluginError> {
        for path in to_remove {
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
