use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use error_stack::{IntoReport, Report, ResultExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    io_utils::{get_confirmation, read_input_from_user, write_string_to_file},
    side::Side,
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) dropbox: PathBuf,
    pub(crate) saves: PathBuf,
    pub(crate) seven_zip_path: PathBuf,
    pub(crate) side: Side,
    pub(crate) player: String,
    pub(crate) turn: u32,

    #[serde(skip)]
    /// The absolute location of the config file
    pub(crate) path: PathBuf,
}

impl Config {
    pub(crate) fn save(&self) -> Result<(), Report<ConfigError>> {
        let config_toml = toml::to_string_pretty(&self)
            .into_report()
            .change_context(ConfigError::Save)?;

        write_string_to_file(config_toml, &self.path).change_context(ConfigError::Save)?;

        Ok(())
    }

    pub(crate) fn from_toml(
        toml_string: &str,
        config_path: &Path,
    ) -> Result<Config, Report<ConfigError>> {
        let mut config: Config = toml::from_str(toml_string)
            .into_report()
            .change_context(ConfigError::Parse)?;
        config.path = config_path.into();
        Ok(config)
    }

    pub(crate) fn write_default_config_file(
        config_path: &Path,
    ) -> Result<Config, Report<ConfigError>> {
        let dropbox_result = dropbox_dir::personal_dir();

        let dropbox = match dropbox_result {
            Ok(dropbox) => dropbox,
            Err(_) => ask_player_for_dropbox_folder()
                .map_err(|_| Report::new(ConfigError::CreateDefaultConfig))
                .attach_printable("Unable to find your dropbox folder")?,
        }
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

        let turn = ask_player_for_a_turn()
            .into_report()
            .change_context(ConfigError::CreateDefaultConfig)
            .attach_printable("Could not read from stdin to ask you for a turn, try again later")?;

        let default_config = Config {
            dropbox,
            saves,
            seven_zip_path,
            side,
            player,
            turn,
            path: config_path.into(),
        };

        default_config
            .save()
            .change_context(ConfigError::CreateDefaultConfig)?;

        println!("New config written to {}", default_config.path.display());

        Ok(default_config)
    }

    pub(crate) fn get(&self, key: Key) -> Setting {
        match key {
            Key::Dropbox => Setting::Dropbox(self.dropbox.clone()),
            Key::Saves => Setting::Saves(self.saves.clone()),
            Key::SevenZipPath => Setting::SevenZipPath(self.seven_zip_path.clone()),
            Key::Side => Setting::Side(self.side),
            Key::Player => Setting::Player(self.player.clone()),
            Key::Turn => Setting::Turn(self.turn),
        }
    }

    pub(crate) fn set(&mut self, key: Key, value: Setting) -> Result<(), Report<ConfigError>> {
        match (key, value) {
            (Key::Dropbox, Setting::Dropbox(value)) => {
                self.dropbox = value;
            }
            (Key::Saves, Setting::Saves(value)) => {
                self.saves = value;
            }
            (Key::SevenZipPath, Setting::SevenZipPath(value)) => {
                self.seven_zip_path = value;
            }
            (Key::Side, Setting::Side(value)) => {
                self.side = value;
            }
            (Key::Player, Setting::Player(value)) => {
                self.player = value;
            }
            (Key::Turn, Setting::Turn(value)) => {
                self.turn = value;
            }
            (Key::Dropbox, _) => return Err(Report::new(ConfigError::InvalidSetting)),
            (Key::Saves, _) => return Err(Report::new(ConfigError::InvalidSetting)),
            (Key::SevenZipPath, _) => return Err(Report::new(ConfigError::InvalidSetting)),
            (Key::Side, _) => return Err(Report::new(ConfigError::InvalidSetting)),
            (Key::Player, _) => return Err(Report::new(ConfigError::InvalidSetting)),
            (Key::Turn, _) => return Err(Report::new(ConfigError::InvalidSetting)),
        }

        self.save()?;
        Ok(())
    }

    pub(crate) fn file_path() -> Result<PathBuf, Report<ConfigError>> {
        Ok(dirs::config_dir()
            .ok_or_else(|| Report::new(ConfigError::UnknownConfigDir))?
            .join("scut")
            .join("config.toml"))
    }

    pub(crate) fn read_config_file(
        config_path: Option<PathBuf>,
    ) -> Result<Config, Report<ConfigError>> {
        let config_path = match config_path {
            Some(config_path) => config_path,
            None => Config::file_path()?,
        };

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
            Ok(ref config_content) => Config::from_toml(config_content, &config_path),
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            toml::to_string_pretty(self).unwrap_or_else(|_| format!("{self:?}"))
        )
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub(crate) enum Key {
    Dropbox,
    Saves,
    SevenZipPath,
    Side,
    Player,
    Turn,
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::Dropbox => write!(f, "dropbox"),
            Key::Saves => write!(f, "saves"),
            Key::SevenZipPath => write!(f, "seven_zip_path"),
            Key::Side => write!(f, "side"),
            Key::Player => write!(f, "player"),
            Key::Turn => write!(f, "turn"),
        }
    }
}

impl FromStr for Key {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "dropbox" => Ok(Self::Dropbox),
            "saves" | "save" => Ok(Self::Saves),
            "seven_zip_path" | "sevenzippath" | "seven-zip-path" | "seven zip path"
            | "sevenzip path" | "sevenzip-path" | "7zpath" | "7z path" | "7z-path" | "7z_path" => {
                Ok(Self::SevenZipPath)
            }
            "side" | "team" => Ok(Self::Side),
            "player" | "name" => Ok(Self::Player),
            "turn" => Ok(Self::Turn),
            _ => Err(ConfigError::InvalidKey),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum Setting {
    Dropbox(PathBuf),
    Saves(PathBuf),
    SevenZipPath(PathBuf),
    Side(Side),
    Player(String),
    Turn(u32),
}

impl Display for Setting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Setting::Dropbox(value) => value.display().fmt(f),
            Setting::Saves(value) => value.display().fmt(f),
            Setting::SevenZipPath(value) => value.display().fmt(f),
            Setting::Side(value) => value.fmt(f),
            Setting::Player(value) => value.fmt(f),
            Setting::Turn(value) => value.fmt(f),
        }
    }
}

impl Setting {
    pub fn new(key: Key, value: String) -> Result<Self, Report<ConfigError>> {
        match key {
            Key::Dropbox => Ok(Setting::Dropbox(value.into())),
            Key::Saves => Ok(Setting::Saves(value.into())),
            Key::SevenZipPath => Ok(Setting::SevenZipPath(value.into())),
            Key::Side => Ok(Setting::Side(
                value
                    .parse()
                    .into_report()
                    .change_context(ConfigError::InvalidSetting)?,
            )),
            Key::Player => Ok(Setting::Player(value)),
            Key::Turn => Ok(Setting::Turn(
                value
                    .parse()
                    .into_report()
                    .change_context(ConfigError::InvalidSetting)?,
            )),
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

fn ask_player_for_a_turn() -> std::io::Result<u32> {
    loop {
        let turn = read_input_from_user("What turn are you on?")?;
        let turn = turn.trim();

        if turn.is_empty() {
            println!("A turn is needed to know which save to download/upload next");
            continue;
        }
        match turn.parse() {
            Ok(turn) => break Ok(turn),
            Err(_) => {
                println!("That's not a valid turn number, please enter a positive integer");
                continue;
            }
        };
    }
}

fn ask_player_for_dropbox_folder() -> std::io::Result<String> {
    println!("Unable to find your dropbox folder");
    println!("You may not have the dropbox client installed. This is required to use scut.");
    println!("If you have installed the dropbox client, then you can let enter your dropbox folder to continue.");
    if !get_confirmation("Would you like to enter your dropbox folder?")? {
        return Ok(String::new());
    }
    loop {
        let dropbox =
            read_input_from_user("Please enter the absolute path to your dropbox folder")?;
        let dropbox = dropbox.trim();

        if dropbox.is_empty() {
            println!("That's not a valid path");
            continue;
        }
        match PathBuf::from_str(dropbox) {
            // TODO: check path exists, is absolute and valid and can be read before returning it!
            Ok(dropbox) => match std::fs::read_dir(&dropbox) {
                Ok(_) => break Ok(dropbox.to_string_lossy().to_string()),
                Err(_) => {
                    println!("scut wasn't able to list the contents of that folder, which means scut is unlikely to work properly.");
                    if get_confirmation("Would you still like to use that folder?")? {
                        break Ok(dropbox.to_string_lossy().to_string());
                    } else {
                        continue;
                    }
                }
            },
            Err(_) => {
                println!("That's not a valid path");
                continue;
            }
        };
    }
}

#[derive(Debug, Error)]
pub(crate) enum ConfigError {
    #[error("Could not read configuration")]
    Read,
    #[error("Could not save configuration")]
    Save,
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
