// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The bee plugin manager.

use log::info;
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};

use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc::{channel, Receiver},
};

const WATCHER_DELAY_SECS: u64 = 1;
const PLUGINS_FOLDER_PATH: &str = "./plugins";

/// Error values raised when a plugin manager operation fails.
#[derive(Debug)]
pub enum Error {
    /// IO errors.
    Io(io::Error),
    /// Notify errors.
    Notify(notify::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => err.fmt(f),
            Self::Notify(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<notify::Error> for Error {
    fn from(err: notify::Error) -> Self {
        Self::Notify(err)
    }
}

/// The plugin manager.
pub struct PluginManager {
    plugins: HashMap<PathBuf, Child>,
    rx: Receiver<DebouncedEvent>,
    _watcher: RecommendedWatcher,
}

impl PluginManager {
    /// Create a new plugin manager.
    pub fn new() -> Result<Self, Error> {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, std::time::Duration::from_secs(WATCHER_DELAY_SECS))?;

        watcher.watch(PLUGINS_FOLDER_PATH, RecursiveMode::Recursive)?;

        Ok(Self {
            plugins: HashMap::new(),
            rx,
            _watcher: watcher,
        })
    }

    fn scan(&mut self) -> Result<(), Error> {
        let entries = std::fs::read_dir(PLUGINS_FOLDER_PATH)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path().canonicalize()?;
            self.load_plugin(&path)?;
        }

        Ok(())
    }

    /// Listen for changes in the plugins folder and loads every new plugin found in it.
    pub fn listen(&mut self) -> Result<(), Error> {
        self.scan()?;

        while let Ok(event) = self.rx.recv() {
            match event {
                DebouncedEvent::Create(path)
                | DebouncedEvent::Write(path)
                | DebouncedEvent::Chmod(path)
                | DebouncedEvent::Rename(_, path) => {
                    let path = path.canonicalize()?;
                    if let Err(err) = self.load_plugin(&path) {
                        info!("plugin \"{}\" could not be loaded: {}", path.display(), err);
                    }
                }
                DebouncedEvent::Rescan => {
                    for (path, child) in self.plugins.drain() {
                        Self::kill_plugin(&path, child);
                    }

                    self.scan()?;
                }
                DebouncedEvent::NoticeWrite(_)
                | DebouncedEvent::NoticeRemove(_)
                | DebouncedEvent::Error(_, _)
                | DebouncedEvent::Remove(_) => (),
            }
        }

        Ok(())
    }

    fn load_plugin(&mut self, path: &Path) -> Result<(), io::Error> {
        self.unload_plugin(path);

        info!("loading plugin \"{}\"", path.display());
        let child = Command::new(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        info!("plugin \"{}\" loaded with PID {}", path.display(), child.id());

        assert!(self.plugins.insert(path.to_owned(), child).is_none());

        Ok(())
    }

    fn unload_plugin(&mut self, path: &Path) {
        if let Some(child) = self.plugins.remove(path) {
            Self::kill_plugin(path, child);
        }
    }

    fn kill_plugin(path: &Path, mut child: Child) {
        info!("killing plugin \"{}\"", path.display());
        match child.kill() {
            Ok(()) => info!("plugin \"{}\" is down", path.display()),
            Err(_) => info!("plugin \"{}\" was already down", path.display()),
        }
    }
}
