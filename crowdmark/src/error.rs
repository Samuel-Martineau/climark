use reqwest::Error;
use std::error::Error as StdError;
use thiserror::Error;
use reqwest::header::InvalidHeaderValue;

#[derive(Error, Debug)]
pub enum CrowdmarkError {
    #[error("Invalid Header Value")]
    HeaderError(#[from] InvalidHeaderValue),
    #[error("Request Error")]
    ReqwestError(#[source] reqwest::Error),
    #[error("Failed to decode JSON")]
    DecodeError(String),
}

impl From<Error> for CrowdmarkError {
    fn from(err: Error) -> Self {
        if err.is_decode() {
            let msg = err.source().map_or_else(
                || "unknown decode error".to_string(),
                std::string::ToString::to_string,
            );
            CrowdmarkError::DecodeError(msg)
        } else {
            CrowdmarkError::ReqwestError(err)
        }
    }
}
