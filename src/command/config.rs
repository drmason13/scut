use clap::Subcommand;
use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::config::{Config, Key, Setting};

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigCmd {
    /// Display information about the current configuration
    Show,

    /// Display the value for a setting
    Get {
        /// which setting to print
        key: Key,
    },

    /// Update a setting
    Set {
        /// which setting to change
        key: Key,

        /// the new value to use
        value: String,
    },
}

impl ConfigCmd {
    pub(crate) fn run(self) -> Result<(), Report<ConfigCmdError>> {
        let config_path = Config::file_path().change_context(ConfigCmdError::Read)?;
        let config = Config::read_config_file().change_context(ConfigCmdError::Read)?;

        match self {
            Self::Show => {
                println!("Config is located at {}", config_path.display());
                println!("{}", config);
            }
            Self::Get { key } => {
                let value = config.get(key);
                println!("{}", value);
            }
            Self::Set { key, value } => {
                let value = normalise(value);
                let mut config = config;
                let value = Setting::new(key, value).change_context(ConfigCmdError::Set)?;
                config.set(key, value).change_context(ConfigCmdError::Set)?;
                println!("config.{key} was updated successfully");
            }
        }

        Ok(())
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum ConfigCmdError {
    #[error("Failed to read config")]
    Read,
    #[error("Failed to update config setting")]
    Set,
}

fn normalise(value: String) -> String {
    let trim_chars: &[_] = &['\'', '"', ' ', '\\'];
    value.trim_matches(trim_chars).to_string()
}
