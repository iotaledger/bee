use std::{error, fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    FlamegraphLayer(io::Error),
    Flamegrapher(FlamegrapherErrorKind),
    LogLayer(LogLayerErrorKind),
}

#[derive(Debug)]
pub enum LogLayerErrorKind {
    Io(io::Error),
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

#[derive(Debug)]
pub enum FlamegraphLayerErrorKind {
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

#[derive(Debug)]
pub enum FlamegrapherErrorKind {
    GraphFileInvalid(PathBuf),
    Inferno(Box<dyn error::Error>),
    Io(io::Error),
    MissingField(String),
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
