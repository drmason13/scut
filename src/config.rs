use std::path::{Path, PathBuf};

use error_stack::{IntoReport, Report, ResultExt};
use serde::{Deserialize, Serialize};

use crate::{
    error::ConfigError,
    side::Side,
    utils::{read_input_from_user, write_string_to_file},
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) dropbox: PathBuf,
    pub(crate) saves: PathBuf,
    pub(crate) seven_zip_path: PathBuf,
    pub(crate) side: Side,
    pub(crate) player: String,
}

impl Config {
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

        let side = ask_player_for_a_side()
            .into_report()
            .change_context(ConfigError::CreateDefaultConfig)
            .attach_printable("Could not read from stdin to ask you for a side, try again later")?;

        let player = ask_player_for_a_name()
            .into_report()
            .change_context(ConfigError::CreateDefaultConfig)
            .attach_printable("Could not read from stdin to ask you for a name, try again later")?;

        let default_config = Config {
            dropbox,
            saves,
            seven_zip_path,
            side,
            player,
        };

        let config_toml = toml::to_string_pretty(&default_config)
            .into_report()
            .change_context(ConfigError::CreateDefaultConfig)?;

        write_string_to_file(config_toml, config_path)
            .change_context(ConfigError::CreateDefaultConfig)?;

        println!("New config written to {}", config_path.display());

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
            Err(e) => Err(e)
                .into_report()
                .change_context(ConfigError::Read)
                .attach_printable("Unexpected error while reading config file"),
            Ok(ref config_content) => {
                let result = toml::from_str(config_content);
                match result {
                    Ok(config) => Ok(config),
                    Err(e) => Err(e).into_report().change_context(ConfigError::Parse),
                }
            }
        }
    }
}

fn ask_player_for_a_side() -> std::io::Result<Side> {
    loop {
        let side = read_input_from_user("What side will you be playing as?")?;

        let side: Result<Side, _> = side.parse();

        match side {
            Ok(side) => break Ok(side),
            Err(_) => {
                println!("The valid sides are 'Axis' and 'Allies', please enter one of those");
                continue;
            }
        };
    }
}

fn ask_player_for_a_name() -> std::io::Result<String> {
    loop {
        let player = read_input_from_user("How do you want to sign your saves?")?;
        let player = player.trim();

        if player.is_empty() {
            println!("A player sign is needed so people know which saves are yours");
            continue;
        }
        break Ok(player.to_string());
    }
}
