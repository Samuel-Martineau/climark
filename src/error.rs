use crowdmark::error::CrowdmarkError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClimarkError {
    #[error(transparent)]
    Crowdmark(#[from] CrowdmarkError),
    #[error("Failed to read stdin")]
    StdinRead(),
    #[error("Could not parse PDF from stdin")]
    PdfParse(),
    #[error("PNG Decode Error")]
    PngDecode(),
    #[error("JPEG Encode Error")]
    JpegEncode(),
}
