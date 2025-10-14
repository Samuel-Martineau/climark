use std::error::Error as StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrowdmarkError {
    #[error("Invalid header value")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Not authenticaed")]
    NotAuthenticated(),
    #[error("Request error")]
    ReqwestError(#[source] reqwest::Error),
    #[error("JSON error")]
    DecodeError(String),
    #[error("Invalid course ID")]
    InvalidCourseID(),
}

impl From<reqwest::Error> for CrowdmarkError {
    fn from(err: reqwest::Error) -> Self {
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

impl From<serde_json::Error> for CrowdmarkError {
    fn from(err: serde_json::Error) -> Self {
        let msg = err.source().map_or_else(
            || "unknown decode error".to_string(),
            std::string::ToString::to_string,
        );
        CrowdmarkError::DecodeError(msg)
    }
}
