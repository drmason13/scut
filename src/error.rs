use thiserror::Error;

#[derive(Debug, Error)]
#[error("SCUT (Strategic Command Utility Tool) has encountered an error")]
pub(crate) struct RuntimeError;

#[derive(Debug, Error)]
#[error("Unable to create a default config file")]
pub(crate) struct WriteDefaultConfigError;

#[derive(Debug, Error)]
#[error("Unable to compress a file")]
pub(crate) struct CompressionError;

#[derive(Debug, Error)]
#[error("No save file found")]
pub(crate) struct NoSaveFileFound;
