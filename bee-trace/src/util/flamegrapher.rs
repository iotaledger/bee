use crate::error::{Error, FlamegrapherErrorKind};

use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Flamegrapher {
    stack_filename: Option<PathBuf>,
    graph_filename: Option<PathBuf>,
}

impl Flamegrapher {
    pub fn new() -> Self {
        Self::default()
    }

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
