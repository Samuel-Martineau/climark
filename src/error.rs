use crowdmark::error::CrowdmarkError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClimarkError {
    #[error(transparent)]
    Crowdmark(#[from] CrowdmarkError),
    #[error("Failed to read stdin")]
    StdinRead(),
    #[error("Could not parse PDF from stdin")]
    PdfParse(),
}
