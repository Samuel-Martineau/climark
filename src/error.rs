use crowdmark::error::CrowdmarkError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClimarkError {
    #[error(transparent)]
    Crowdmark(#[from] CrowdmarkError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error("Could not parse PDF from stdin")]
    PdfParse,
    #[error("Failed to read stdin")]
    StdinRead,
}
