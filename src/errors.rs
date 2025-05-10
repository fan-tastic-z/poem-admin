use thiserror::Error;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct Error(pub String);
