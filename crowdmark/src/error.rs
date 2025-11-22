use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrowdmarkError {
    #[error("Invalid header value")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Not authenticated")]
    NotAuthenticated(String),
    #[error("Request error")]
    Reqwest(#[source] reqwest::Error),
    #[error("JSON error")]
    Decode(String),
    #[error("Invalid course ID")]
    InvalidCourseID(),
    #[error("Invalid assessment ID")]
    InvalidAssessmentID(),
    #[error("Too many pages submitted")]
    TooManyPages(),
    #[error("Regex compile error")]
    Regex(#[from] regex::Error),
    #[error("Invalid S3 Policy Response")]
    S3Policy(),
    #[error("Failed to upload to S3")]
    S3Upload(String),
    #[error("Failed to upload to Crowdmark assignment")]
    AssignmentUpload(String),
    #[error("Failed to submit Crowdmark assignment")]
    AssignmentSubmit(String),
    #[error("Failed to login")]
    Login(),
}

impl From<reqwest::Error> for CrowdmarkError {
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
    fn from(err: serde_json::Error) -> Self {
        let msg = err.to_string();
        CrowdmarkError::Decode(msg)
    }
}
