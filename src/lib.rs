use thiserror::Error;

pub mod cli;
pub mod config;
pub mod domain;
pub mod input;
pub mod output;
pub mod utils;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct Error(pub String);
