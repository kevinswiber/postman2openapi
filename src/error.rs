use failure::Fail;
use openapi::Error as OpenApiError;
use serde_json::Error as JsonError;
use std::io::Error as IoError;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Io(IoError),
    #[fail(display = "{}", _0)]
    OpenApi(OpenApiError),
    #[fail(display = "{}", _0)]
    Serialize(JsonError),
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
    }
}

impl From<OpenApiError> for Error {
    fn from(e: OpenApiError) -> Self {
        Error::OpenApi(e)
    }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Self {
        Error::Serialize(e)
    }
}
