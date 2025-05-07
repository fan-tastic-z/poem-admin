use thiserror::Error;

pub mod config;
pub mod server;
pub mod start;
pub mod utils;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct Error(pub String);
