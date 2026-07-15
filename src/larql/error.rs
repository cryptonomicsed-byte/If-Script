use thiserror::Error;

#[derive(Debug, Error)]
pub enum LarqlError {
    #[error("LARQL parse error: {0}")]
    Parse(String),
    #[error("unknown Action Vessel: {0}")]
    UnknownVessel(String),
    #[error("no Odù found for {0}")]
    NotFound(String),
}
