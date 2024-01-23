use anyhow::Context;
use scut_core::{
    interface::{
        config::{toml_file::TomlFileConfig, ConfigService},
        file_system::local_file_system::LocalFileSystem,
    },
    Config,
};
use std::path::PathBuf;

use crate::ui::TauriWindow;

fn get_config(config_service: &mut dyn ConfigService) -> anyhow::Result<Config> {
    match config_service.load().context("failed to load config") {
        Ok(Some(config)) => Ok(config),
        Ok(None) => Ok(config_service
            .init_config()
            .context("failed to create a new config")?),
        Err(e) => Err(e),
    }
}

/// Instantiate the interfaces and their dependencies for a Config and ConfigService
pub(crate) fn ready_config(
    config_option: Option<PathBuf>,
) -> anyhow::Result<(Config, Box<dyn ConfigService>)> {
    let file_system = Box::new(LocalFileSystem::new());
    let user_interaction = Box::new(TauriWindow);

    let config_location = config_option.unwrap_or(TomlFileConfig::default_location()?);
    let mut config_service = TomlFileConfig::new(config_location, file_system, user_interaction);
    Ok((get_config(&mut config_service)?, Box::new(config_service)))
}
