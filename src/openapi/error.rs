//! Error types

use failure::Fail;
use semver::{SemVerError, Version};
use serde_json::Error as JsonError;
use serde_yaml::Error as YamlError;
use std::io::Error as IoError;

/// errors that openapi functions may return
#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Io(IoError),
    #[fail(display = "{}", _0)]
    Yaml(YamlError),
    #[fail(display = "{}", _0)]
    Serialize(JsonError),
    #[fail(display = "{}", _0)]
    SemVerError(SemVerError),
    #[fail(display = "Unsupported spec file version ({})", _0)]
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
