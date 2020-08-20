use thiserror::Error;

use semver::{SemVerError, Version};
use serde_json::Error as JsonError;
use serde_yaml::Error as YamlError;
use std::io::Error as IoError;

use super::openapi;

#[derive(Error, Debug)]
pub enum OpenApiError {
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

impl From<openapi::Error> for OpenApiError {
    fn from(e: openapi::Error) -> Self {
        match e {
            openapi::Error::Io(err) => OpenApiError::Io(err),
            openapi::Error::Yaml(err) => OpenApiError::Yaml(err),
            openapi::Error::Serialize(err) => OpenApiError::Serialize(err),
            openapi::Error::SemVerError(err) => OpenApiError::SemVerError(err),
            openapi::Error::UnsupportedSpecFileVersion(err) => {
                OpenApiError::UnsupportedSpecFileVersion(err)
            }
        }
    }
}
