use std::io;
use std::time::SystemTimeError;
use actix_web::http::StatusCode;
use awc::error::{JsonPayloadError, SendRequestError, PayloadError};
use serde_json::Value;
use zip::result::ZipError;

#[derive(Debug)]
pub enum PreviewerError {
    RequestError(SendRequestError),
    JsonPayloadError(JsonPayloadError),
    PayloadError(PayloadError),
    ZipError(ZipError),
    IOError(io::Error),
    SystemTimeError(SystemTimeError),
    StatusError { url: String, status_code: StatusCode },
    ResponseContentError(String, Value),
    PatternNotFound(String)
}

impl From<SendRequestError> for PreviewerError {
    fn from(item: SendRequestError) -> Self {
        Self::RequestError(item)
    }
}

impl From<JsonPayloadError> for PreviewerError {
    fn from(item: JsonPayloadError) -> Self {
        Self::JsonPayloadError(item)
    }
}

impl From<PayloadError> for PreviewerError {
    fn from(item: PayloadError) -> Self {
        Self::PayloadError(item)
    }
}

impl From<ZipError> for PreviewerError {
    fn from(item: ZipError) -> Self {
        Self::ZipError(item)
    }
}

impl From<io::Error> for PreviewerError {
    fn from(item: io::Error) -> Self {
        Self::IOError(item)
    }
}

impl From<SystemTimeError> for PreviewerError {
    fn from(item: SystemTimeError) -> Self {
        Self::SystemTimeError(item)
    }
}
