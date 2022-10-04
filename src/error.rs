use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ConfigError {
    #[error("Could not read configuration")]
    Read,
    #[error("Unable to locate config directory")]
    UnknownConfigDir,
    #[error("Could not parse configuration file")]
    Parse,
    #[error("Could not create default configuration file")]
    CreateDefaultConfig,
    #[error("Setting does not exist")]
    InvalidKey,
    #[error("Invalid value for setting")]
    InvalidSetting,
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
