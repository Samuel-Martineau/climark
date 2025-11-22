use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrowdmarkError {
    #[error("Invalid header value")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Not authenticated")]
    NotAuthenticated(String),
    #[error("Request error")]
    ReqwestError(#[source] reqwest::Error),
    #[error("JSON error")]
    DecodeError(String),
    #[error("Invalid course ID")]
    InvalidCourseID(),
    #[error("Invalid assessment ID")]
    InvalidAssessmentID(),
    #[error("Too many pages submitted")]
    TooManyPages(),
    #[error("Regex compile error")]
    RegexError(#[from] regex::Error),
    #[error("Invalid S3 Policy Response")]
    S3PolicyError(),
    #[error("Failed to upload to S3")]
    S3UploadError(String),
    #[error("Failed to upload to Crowdmark assignment")]
    AssignmentUploadError(String),
    #[error("Failed to submit Crowdmark assignment")]
    AssignmentSubmitError(String),
}

impl From<reqwest::Error> for CrowdmarkError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_decode() {
            let msg = err.to_string();
            CrowdmarkError::DecodeError(msg)
        } else {
            CrowdmarkError::ReqwestError(err)
        }
    }
}

impl From<serde_json::Error> for CrowdmarkError {
    fn from(err: serde_json::Error) -> Self {
        let msg = err.to_string();
        CrowdmarkError::DecodeError(msg)
    }
}
