use clap::Subcommand;
use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::{
    config::{Config, Key, Setting},
    io_utils::{get_confirmation, wait_for_user_before_close},
};

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

    /// Edit the config file in the system editor
    Edit,
}

impl ConfigCmd {
    pub(crate) fn run(self, config: Config) -> Result<(), Report<ConfigCmdError>> {
        match self {
            Self::Show => {
                println!("Config is located at {}", config.path.display());
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
            Self::Edit => {
                let new_string = loop {
                    match edit::edit(&config.to_string()) {
                        Ok(new_string) => break new_string,
                        Err(io_err) if io_err.kind() == std::io::ErrorKind::InvalidData => {
                            println!("The edited config was not valid UTF-8");
                            println!("Your changes have not been saved.");

                            if get_confirmation("Would you like to try and edit the config again?")
                                .unwrap_or(false)
                            {
                                continue;
                            } else {
                                wait_for_user_before_close("User has abandoned editing the config. Exiting.");
                            }
                        },
                        Err(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
                            println!("Unable to find an editor to edit the config");
                            wait_for_user_before_close("You can edit the config from the commandline using `scut config set KEY VALUE`");
                            return Ok(())
                        },
                        Err(e) => {
                            return Err(Report::new(e)
                            .attach_printable("Unknown error while attempting to open an editor and edit the config")
                            .change_context(ConfigCmdError::Edit))
                        }
                    }
                };

                let new_config = loop {
                    match Config::from_toml(&new_string, &config.path) {
                        Ok(config) => break config,
                        Err(e) => {
                            println!("Invalid config: {e}");
                            println!("Your changes have not been saved.");

                            if get_confirmation("Would you like to try and edit the config again?")
                                .unwrap_or(false)
                            {
                                continue;
                            } else {
                                wait_for_user_before_close(
                                    "User has abandoned editing the config. Exiting.",
                                );
                                return Ok(());
                            }
                        }
                    }
                };

                new_config.save().change_context(ConfigCmdError::Edit)?;

                println!("Config was updated successfully");
            }
        }

        wait_for_user_before_close("");
        Ok(())
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum ConfigCmdError {
    #[error("Failed to update config setting")]
    Set,
    #[error("Failed to edit the config directly")]
    Edit,
}

fn normalise(value: String) -> String {
    let trim_chars: &[char] = &['\'', '"', ' ', '\\'];
    value.trim_matches(trim_chars).to_string()
}
