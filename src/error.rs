use thiserror::Error;

#[derive(Debug, Error)]
#[error("Unable to locate config directory")]
pub struct UnknownConfigDirError;

#[derive(Debug, Error)]
#[error("Strategic Command cp has encountered a fatal error")]
pub struct RuntimeError;

#[derive(Debug, Error)]
#[error("Unable to create a default config file")]
pub struct WriteDefaultConfigError;

#[derive(Debug, Error)]
#[error("Unable to compress a file")]
pub struct CompressionError;
