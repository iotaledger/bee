// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{PluginError, PluginId, PluginManager, UniqueId};

use bee_event_bus::EventBus;
use tokio::{
    process::Command,
    time::{sleep, Duration},
};

use std::{collections::HashMap, path::PathBuf, sync::Arc, time::SystemTime};

struct PluginInfo {
    id: PluginId,
    last_write: SystemTime,
}

pub struct Hotloader {
    plugins_info: HashMap<PathBuf, PluginInfo>,
    manager: PluginManager,
}

impl Hotloader {
    pub fn new(event_bus: Arc<EventBus<'static, UniqueId>>) -> Self {
        Self {
            plugins_info: HashMap::new(),
            manager: PluginManager::new(event_bus),
        }
    }

    pub async fn run(mut self) -> Result<(), PluginError> {
        loop {
            let mut dir = tokio::fs::read_dir("./plugins").await?;

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
                            self.manager.unload_plugin(info.id).await?;
                            let command = Command::new(path);
                            info.id = self.manager.load_plugin(command).await?;
                            info.last_write = last_write;
                        }
                    }
                    // The file no longer exists
                    None => {
                        to_delete.push(path.clone());
                        self.manager.unload_plugin(info.id).await?;
                    }
                }
            }

            // remove info for files that no longer exist
            for path in to_delete {
                self.plugins_info.remove(&path);
            }

            // Load plugins whose files did not exist before.
            for (path, last_write) in last_writes {
                let command = Command::new(&path);
                let id = self.manager.load_plugin(command).await?;
                self.plugins_info.insert(path, PluginInfo { id, last_write });
            }

            sleep(Duration::from_secs(1)).await;
        }
    }
}
