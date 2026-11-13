use std::fmt;
use std::io;

/// Errors produced during governor operations and hardware telemetry access.
#[derive(Debug)]
pub enum GovernorError {
    /// Failure during file or sysfs I/O.
    Io(io::Error),
    /// Failure to parse numerical or textual telemetry node.
    Parse(String),
    /// Sysfs interface not supported or not found.
    UnsupportedSysfs(String),
}

impl fmt::Display for GovernorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Parse(msg) => write!(f, "Parse error: {msg}"),
            Self::UnsupportedSysfs(path) => write!(f, "Unsupported sysfs interface: {path}"),
        }
    }
}

impl std::error::Error for GovernorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Parse(_) | Self::UnsupportedSysfs(_) => None,
        }
    }
}

impl From<io::Error> for GovernorError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
