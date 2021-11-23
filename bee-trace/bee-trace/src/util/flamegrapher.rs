// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::error::{Error, FlamegrapherErrorKind};

use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

/// Helper struct that uses [`inferno`] internally to programatically produce a flamegraph from a folded
/// stack trace file.
#[derive(Default)]
pub struct Flamegrapher {
    stack_filename: Option<PathBuf>,
    graph_filename: Option<PathBuf>,
}

impl Flamegrapher {
    /// Creates a new [`Flamegrapher`] with no associated files.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a [`Flamegrapher`] with the given folded stack file name.
    ///
    /// This file will be given the extension of `.folded`.
    ///
    /// # Errors
    /// This method may fail in the following ways:
    ///  - The given stack file does not exist.
    pub fn with_stack_file<P: AsRef<Path>>(mut self, stack_filename: P) -> Result<Self, Error> {
        let stack_filename = stack_filename.as_ref().to_path_buf();

        if !stack_filename.exists() {
            return Err(Error::Flamegrapher(FlamegrapherErrorKind::StackFileNotFound(
                stack_filename,
            )));
        }

        self.stack_filename = Some(stack_filename);
        Ok(self)
    }

    /// Returns a [`Flamegrapher`] with the given flamegraph file name.
    ///
    /// This file will be given the extension of `.svg`.
    ///
    /// # Errors
    /// This method may fail in the following ways:
    ///  - The given graph filename is invalid (it does not belong to a directory that exists).
    pub fn with_graph_file<P: AsRef<Path>>(mut self, graph_filename: P) -> Result<Self, Error> {
        let graph_filename = graph_filename.as_ref().with_extension("svg");

        match graph_filename.parent() {
            Some(directory) if !directory.is_dir() => {
                return Err(Error::Flamegrapher(FlamegrapherErrorKind::GraphFileInvalid(
                    graph_filename,
                )));
            }
            _ => {}
        }

        self.graph_filename = Some(graph_filename);
        Ok(self)
    }

    /// Uses [`inferno`] to generate a flamegraph from the given folded stack file, and writes it to the given
    /// output image file.
    ///
    /// # Errors
    /// This method may fail in the following ways:
    ///  - This [`Flamegrapher`] does not have a stack or graph file associated with it.
    ///  - An error was encountered when opening the folded stack file for reading.
    ///  - An error was encountered when creating the graph file.
    pub fn write_flamegraph(&self) -> Result<(), Error> {
        let stack_filename = self
            .stack_filename
            .as_ref()
            .ok_or_else(|| Error::Flamegrapher(FlamegrapherErrorKind::MissingField("stack_filename".to_string())))?;

        let graph_filename = self
            .graph_filename
            .as_ref()
            .ok_or_else(|| Error::Flamegrapher(FlamegrapherErrorKind::MissingField("graph_filename".to_string())))?;

        let stack_file = File::open(stack_filename).map_err(|err| Error::Flamegrapher(err.into()))?;
        let reader = BufReader::new(stack_file);

        let graph_file = File::create(graph_filename).map_err(|err| Error::Flamegrapher(err.into()))?;
        let writer = BufWriter::new(graph_file);

        let mut graph_options = inferno::flamegraph::Options::default();
        inferno::flamegraph::from_reader(&mut graph_options, reader, writer)
            .map_err(|err| Error::Flamegrapher(FlamegrapherErrorKind::Inferno(Box::new(err))))?;

        Ok(())
    }
}
