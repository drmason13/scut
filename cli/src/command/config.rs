use anyhow::Context;
use clap::Subcommand;

use crate::io_utils::{get_confirmation, wait_for_user_before_close};

use scut_core::interface::ConfigPersistence;
use scut_core::{Config, Key, Setting};

/// Read or modify the current configuration file
///
/// The configuration file is used to decide what to name your saves\n
/// when uploading, and which saves to download
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
    pub(crate) fn run(self, mut config: Box<dyn ConfigPersistence>) -> anyhow::Result<()> {
        match self {
            Self::Show => {
                println!("Config is located at {}", config.display_location());
                println!("{}", config.display());
            }
            Self::Get { key } => {
                let config = config.load()?;
                let value = config.get(key);
                println!("{value}");
            }
            Self::Set { key, value } => {
                let value = normalise(value);
                let mut config = config.load()?;
                let value = Setting::new(key, value)
                    .with_context(|| format!("failed to set config.{key}"))?;
                config.set(key, value)?;

                println!("config.{key} was updated successfully");
            }
            Self::Edit => {
                let new_string = loop {
                    match edit::edit(config.display().to_string()) {
                        Ok(new_string) => break new_string,
                        Err(io_err) if io_err.kind() == std::io::ErrorKind::InvalidData => {
                            println!("The edited config was not valid UTF-8");
                            println!("Your changes have not been saved.");

                            if get_confirmation("Would you like to try and edit the config again?")
                            {
                                continue;
                            } else {
                                wait_for_user_before_close("Config was not updated. Exiting.");
                            }
                        }
                        Err(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
                            println!("Unable to find an editor to edit the config");
                            wait_for_user_before_close("You can edit the config from the commandline using `scut config set KEY VALUE`");
                            return Ok(());
                        }
                        Err(e) => {
                            return Err(e).context("failed to open an editor to edit the config")
                        }
                    }
                };

                let new_config = loop {
                    match config.deserialize(new_string.as_str()) {
                        Ok(config) => break config,
                        Err(e) => {
                            println!("Invalid config: {e}");
                            println!("Your changes have not been saved.");

                            if get_confirmation("Would you like to try and edit the config again?")
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

                config
                    .save(new_config)
                    .context("failed to save changes to config")?;

                println!("Config was updated successfully");
            }
        }

        wait_for_user_before_close("");
        Ok(())
    }
}

fn normalise(value: String) -> String {
    let trim_chars: &[char] = &['\'', '"', ' ', '\t', '\n', '\\'];
    value.trim_matches(trim_chars).to_string()
}
