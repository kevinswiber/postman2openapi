//! Error types

use serde_json::Error as JsonError;
#[cfg(not(target_arch = "wasm32"))]
use serde_yaml::Error as YamlError;
use std::io::Error as IoError;
use thiserror::Error;

/// errors that openapi functions may return
#[cfg(not(target_arch = "wasm32"))]
#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(IoError),
    #[error("{0}")]
    Yaml(YamlError),
    #[error("{0}")]
    Serialize(JsonError),
}

#[cfg(target_arch = "wasm32")]
#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(IoError),
    #[error("{0}")]
    Serialize(JsonError),
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
    }
}

#[cfg(not(target_arch = "wasm32"))]
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
