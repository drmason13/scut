mod edit;
use edit::edit;

use anyhow::Context;
use clap::{Args, Subcommand};

use scut_core::interface::config::ConfigService;
use scut_core::interface::UserInteraction;
use scut_core::{Config, Key, Setting};
use tracing::{debug, instrument};

/// Read or modify the current configuration file
///
/// The configuration file is used to decide what to name your saves\n
/// when uploading, and which saves to download
#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub(crate) sub_cmd: ConfigSubcommand,
}

/// Config Subcommands
#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
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

/// Configure scut's config file
#[instrument(skip_all, ret, err)]
pub fn run(
    args: ConfigArgs,
    config: Config,
    mut config_service: Box<dyn ConfigService>,
    mut ui: Box<dyn UserInteraction>,
) -> anyhow::Result<()> {
    match args.sub_cmd {
        ConfigSubcommand::Show => {
            debug!(subcommand = "show");
            ui.message(&format!(
                "Config is located at {}",
                config_service.location()?
            ));
            ui.message(&config_service.serialize(&config)?);
        }
        ConfigSubcommand::Get { key } => {
            debug!(subcommand = "get");
            let value = config.get(key);
            ui.message(&format!("{value}"));
        }
        ConfigSubcommand::Set { key, value } => {
            debug!(subcommand = "set", %key, %value);
            let value = normalise(value);
            let setting = Setting::new(key, value)
                .with_context(|| format!("Invalid value for config.{key}"))?;
            let updated_config = config.set(setting);

            config_service
                .save(&updated_config)
                .with_context(|| "failed to save config to disk")?;

            ui.message(&format!("config.{key} was updated successfully"));
        }
        ConfigSubcommand::Edit => {
            debug!(subcommand = "edit");
            edit(config, config_service, &mut *ui)?;
        }
    }

    ui.wait_for_user_before_close("");
    Ok(())
}

fn normalise(value: String) -> String {
    let trim_chars: &[char] = &['\'', '"', ' ', '\t', '\n', '\\'];
    value.trim_matches(trim_chars).to_string()
}
