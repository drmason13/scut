use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use error_stack::{IntoReport, Report, ResultExt};
use serde::{Deserialize, Serialize};

use crate::{error::ConfigError, fs::write_string_to_file};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) dropbox: PathBuf,
    pub(crate) saves: PathBuf,
    pub(crate) seven_zip_path: PathBuf,
    pub(crate) side: Side,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Side {
    Axis,
    Allies,
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Axis => write!(f, "Axis"),
            Self::Allies => write!(f, "Allies"),
        }
    }
}

impl Config {
    pub(crate) fn dropbox(&self) -> &Path {
        &self.dropbox
    }
    pub(crate) fn saves(&self) -> &Path {
        &self.saves
    }
    pub(crate) fn seven_zip_path(&self) -> &Path {
        &self.seven_zip_path
    }

    pub(crate) fn write_default_config_file(
        config_path: &Path,
    ) -> Result<Config, Report<ConfigError>> {
        let dropbox = dropbox_dir::personal_dir()
            .map_err(|_| Report::new(ConfigError::CreateDefaultConfig))
            .attach_printable("Unable to find your dropbox folder")?
            .into();
        let home = dirs::home_dir()
            .ok_or_else(|| Report::new(ConfigError::CreateDefaultConfig))
            .attach_printable("Unable to find your documents folder")?;
        let saves = home.join(
            r#"Documents\My Games\Strategic Command WWII - World at War\Multiplayer\Hotseat"#,
        );
        let seven_zip_path = PathBuf::from(r#"C:\Program Files\7-Zip\"#);

        let default_config = Config {
            dropbox,
            saves,
            seven_zip_path,
            side: Side::Allies,
        };

        let config_toml = toml::to_string_pretty(&default_config)
            .into_report()
            .change_context(ConfigError::CreateDefaultConfig)?;

        write_string_to_file(config_toml, config_path)
            .change_context(ConfigError::CreateDefaultConfig)?;

        Ok(default_config)
    }

    pub(crate) fn read_config_file() -> Result<Config, Report<ConfigError>> {
        // read config
        let config_path = dirs::config_dir()
            .ok_or_else(|| Report::new(ConfigError::UnknownConfigDir))?
            .join("scut")
            .join("config.toml");

        let file_result = std::fs::read_to_string(&config_path);

        match file_result {
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                eprintln!(
                    "No config file found.\nWriting default config file to {}",
                    config_path.display()
                );
                let default_config = Config::write_default_config_file(&config_path)
                    .attach_printable(
                        "Attempted to create a default config for you but there was a problem",
                    )?;
                Ok(default_config)
            }
            Err(e) => Err(Report::new(ConfigError::Io(e))
                .attach_printable("Unexpected error while reading config file")),
            Ok(ref config_content) => toml::from_str(config_content)
                .map_err(|_| ConfigError::Parse)
                .into_report(),
        }
    }
}
