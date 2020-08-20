//! Error types

use semver::{SemVerError, Version};
use serde_json::Error as JsonError;
use serde_yaml::Error as YamlError;
use std::io::Error as IoError;
use thiserror::Error;

/// errors that openapi functions may return
#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(IoError),
    #[error("{0}")]
    Yaml(YamlError),
    #[error("{0}")]
    Serialize(JsonError),
    #[error("{0}")]
    SemVerError(SemVerError),
    #[error("Unsupported spec file version ({0})")]
    UnsupportedSpecFileVersion(Version),
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
    }
}

impl From<YamlError> for Error {
    fn from(e: YamlError) -> Self {
        Error::Yaml(e)
    }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Self {
        Error::Serialize(e)
    }
}

impl From<SemVerError> for Error {
    fn from(e: SemVerError) -> Self {
        Error::SemVerError(e)
    }
}
