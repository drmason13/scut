use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Side;

mod key;
mod setting;

pub use key::Key;
pub use setting::Setting;

// TODO: implement config extensions for storing implementation specific settings
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub saves: PathBuf,
    pub side: Side,
    pub player: String,
    pub turn: u32,
    pub dropbox: PathBuf,
    pub seven_zip_path: PathBuf,
    #[serde(default)]
    pub team_names: TeamNames,
}

/// Used to determine how to parse saves at runtime
///
/// Play with "Axis" first and "Allies" second by default, but you can play with whatever team names you like!
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamNames {
    pub axis: String,
    pub allies: String,
}

impl TeamNames {
    pub fn new((axis, allies): (String, String)) -> Self {
        TeamNames { axis, allies }
    }
}

impl Default for TeamNames {
    fn default() -> Self {
        TeamNames::new(("Axis".to_string(), "Allies".to_string()))
    }
}

impl Config {
    pub fn get(&self, key: Key) -> Setting {
        match key {
            Key::Dropbox => Setting::Dropbox(self.dropbox.clone()),
            Key::Saves => Setting::Saves(self.saves.clone()),
            Key::SevenZipPath => Setting::SevenZipPath(self.seven_zip_path.clone()),
            Key::Side => Setting::Side(self.side),
            Key::Player => Setting::Player(self.player.clone()),
            Key::Turn => Setting::Turn(self.turn),
        }
    }

    pub fn set(mut self, key: Key, value: Setting) -> anyhow::Result<Config> {
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
            (key @ Key::Dropbox, _) => anyhow::bail!("invalid setting for key {key}"),
            (key @ Key::Saves, _) => anyhow::bail!("invalid setting for key {key}"),
            (key @ Key::SevenZipPath, _) => anyhow::bail!("invalid setting for key {key}"),
            (key @ Key::Side, _) => anyhow::bail!("invalid setting for key {key}"),
            (key @ Key::Player, _) => anyhow::bail!("invalid setting for key {key}"),
            (key @ Key::Turn, _) => anyhow::bail!("invalid setting for key {key}"),
        }

        Ok(self)
    }
}
