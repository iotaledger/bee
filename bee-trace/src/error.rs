// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{error, fmt, io, path::PathBuf};

/// Error `enum` containing variants for any errors that could potentially occur in this crate.
#[derive(Debug)]
pub enum Error {
    /// Error originating from the [`FlamegraphLayer`](crate::subscriber::layer::FlamegraphFilteredLayer).
    FlamegraphLayer(io::Error),
    /// Error originating from the [`Flamegrapher`](crate::util::Flamegrapher).
    Flamegrapher(FlamegrapherErrorKind),
    /// Error originating from the [`LogLayer`](crate::subscriber::layer::LogLayer).
    LogLayer(LogLayerErrorKind),
}

/// An error originating from the [`LogLayer`](crate::subscriber::layer::LogLayer).
#[derive(Debug)]
pub enum LogLayerErrorKind {
    /// Encountered an [`io::Error`].
    Io(io::Error),
    /// Error setting the default logger/subscriber.
    SetLogger(log::SetLoggerError),
}

impl fmt::Display for LogLayerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Io(err) => write!(f, "{}", err),
            Self::SetLogger(err) => write!(f, "{}", err),
        }
    }
}

impl From<io::Error> for LogLayerErrorKind {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<log::SetLoggerError> for LogLayerErrorKind {
    fn from(err: log::SetLoggerError) -> Self {
        Self::SetLogger(err)
    }
}

/// An error originating from the [`FlamegraphLayer`](crate::subscriber::layer::FlamegraphFilteredLayer).
#[derive(Debug)]
pub enum FlamegraphLayerErrorKind {
    /// Encountered an [`io::Error`].
    Io(io::Error),
}

impl fmt::Display for FlamegraphLayerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Io(err) => write!(f, "{}", err),
        }
    }
}

impl From<io::Error> for FlamegraphLayerErrorKind {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// An error originating from the [`Flamegrapher`](crate::util::Flamegrapher).
#[derive(Debug)]
pub enum FlamegrapherErrorKind {
    /// The provided graph file is invalid.
    GraphFileInvalid(PathBuf),
    /// Usage of [`inferno`] resulted in an error.
    Inferno(Box<dyn error::Error>),
    /// Encountered an [`io::Error`].
    Io(io::Error),
    /// The [`Flamegrapher`](crate::util::Flamegrapher) is missing a required field.
    MissingField(String),
    /// Could not find the provided folded stack file.
    StackFileNotFound(PathBuf),
}

impl fmt::Display for FlamegrapherErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::GraphFileInvalid(path) => write!(
                f,
                "invalid flamegraph file path: directory {} does not exist",
                path.to_string_lossy()
            ),
            Self::Inferno(err) => write!(f, "{}", err),
            Self::Io(err) => write!(f, "{}", err),
            Self::MissingField(field) => write!(f, "flamegrapher builder missing field: {}", field),
            Self::StackFileNotFound(path) => write!(f, "folded stack file {} does not exist", path.to_string_lossy()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::FlamegraphLayer(err) => write!(f, "{}", err),
            Self::Flamegrapher(err) => write!(f, "{}", err),
            Self::LogLayer(err) => write!(f, "{}", err),
        }
    }
}

impl From<Box<dyn error::Error>> for FlamegrapherErrorKind {
    fn from(err: Box<dyn error::Error>) -> Self {
        Self::Inferno(err)
    }
}

impl From<io::Error> for FlamegrapherErrorKind {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self {
            Self::FlamegraphLayer(err) => Some(err),
            Self::Flamegrapher(FlamegrapherErrorKind::Io(err)) => Some(err),
            Self::LogLayer(LogLayerErrorKind::Io(err)) => Some(err),
            Self::LogLayer(LogLayerErrorKind::SetLogger(err)) => Some(err),
            _ => None,
        }
    }
}
