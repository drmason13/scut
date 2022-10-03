use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ConfigError {
    #[error("Unable to locate config directory")]
    UnknownConfigDir,
    #[error(transparent)]
    Io(std::io::Error),
    #[error("Could not parse configuration file")]
    Parse,
    #[error("Could not create default configuration file")]
    CreateDefaultConfig,
}

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
