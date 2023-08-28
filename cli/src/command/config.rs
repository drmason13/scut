use anyhow::Context;
use clap::Subcommand;

use scut_core::interface::config::ConfigService;
use scut_core::interface::UserInteraction;
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
    pub(crate) fn run(
        self,
        config: Config,
        mut config_service: Box<dyn ConfigService>,
        mut ui: Box<dyn UserInteraction>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Show => {
                ui.message(&format!(
                    "Config is located at {}",
                    config_service.location()?
                ));
                ui.message(&config_service.serialize(&config)?);
            }
            Self::Get { key } => {
                let value = config.get(key);
                println!("{value}");
            }
            Self::Set { key, value } => {
                let value = normalise(value);
                let value = Setting::new(key, value)
                    .with_context(|| format!("failed to set config.{key}"))?;
                config.set(key, value)?;

                println!("config.{key} was updated successfully");
            }
            Self::Edit => {
                let new_string = loop {
                    match edit::edit(config_service.serialize(&config)?) {
                        Ok(new_string) => break new_string,
                        Err(io_err) if io_err.kind() == std::io::ErrorKind::InvalidData => {
                            println!("The edited config was not valid UTF-8");
                            println!("Your changes have not been saved.");

                            if ui.confirm(
                                "Would you like to try and edit the config again?",
                                Some(true),
                            ) {
                                continue;
                            } else {
                                ui.wait_for_user_before_close("Config was not updated. Exiting.");
                            }
                        }
                        Err(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
                            ui.message("Unable to find an editor to edit the config");
                            ui.wait_for_user_before_close("You can edit the config from the commandline using `scut config set KEY VALUE`");
                            return Ok(());
                        }
                        Err(e) => {
                            return Err(e).context("failed to open an editor to edit the config")
                        }
                    }
                };

                let new_config = loop {
                    match config_service.deserialize(new_string.as_str()) {
                        Ok(config) => break config,
                        Err(e) => {
                            println!("Invalid config: {e}");
                            println!("Your changes have not been saved.");

                            if ui.confirm("Would you like to try and edit the config again?", None)
                            {
                                continue;
                            } else {
                                ui.wait_for_user_before_close(
                                    "User has abandoned editing the config. Exiting.",
                                );
                                return Ok(());
                            }
                        }
                    }
                };

                config_service
                    .save(&new_config)
                    .context("failed to save changes to config")?;

                println!("Config was updated successfully");
            }
        }

        ui.wait_for_user_before_close("");
        Ok(())
    }
}

fn normalise(value: String) -> String {
    let trim_chars: &[char] = &['\'', '"', ' ', '\t', '\n', '\\'];
    value.trim_matches(trim_chars).to_string()
}
