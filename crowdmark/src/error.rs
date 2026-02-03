use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CrowdmarkError {
    #[error("Failed to submit Crowdmark assessment")]
    AssessmentSubmit(String),
    #[error("Failed to upload to Crowdmark assessment")]
    AssessmentUpload(String),
    #[error("JSON error")]
    Decode(String),
    #[error("Invalid assessment ID")]
    InvalidAssessmentID(),
    #[error("Invalid course ID")]
    InvalidCourseID(),
    #[error("Invalid header value")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Tokio join error")]
    Join(#[from] tokio::task::JoinError),
    #[error("Failed to login")]
    Login(),
    #[error("Not authenticated")]
    NotAuthenticated(String),
    #[error("Regex compile error")]
    Regex(#[from] regex_lite::Error),
    #[error("Request error")]
    Reqwest(#[source] reqwest::Error),
    #[error("Invalid S3 Policy Response")]
    S3Policy(),
    #[error("Failed to upload to S3")]
    S3Upload(String),
    #[error("Too many pages submitted")]
    TooManyPages(),
}

impl From<reqwest::Error> for CrowdmarkError {
    #[inline]
    fn from(err: reqwest::Error) -> Self {
        if err.is_decode() {
            let msg = err.to_string();
            CrowdmarkError::Decode(msg)
        } else {
            CrowdmarkError::Reqwest(err)
        }
    }
}

impl From<serde_json::Error> for CrowdmarkError {
    #[inline]
    fn from(err: serde_json::Error) -> Self {
        let msg = err.to_string();
        CrowdmarkError::Decode(msg)
    }
}
