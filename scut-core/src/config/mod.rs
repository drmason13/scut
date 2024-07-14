use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Side;

mod key;
mod setting;

pub use key::Key;
pub use setting::Setting;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub saves: PathBuf,
    pub side: Side,
    pub player: String,
    #[serde(default)]
    pub turn: Option<u32>,
    #[serde(default)]
    pub solo: Option<bool>,

    pub dropbox: PathBuf,
    pub seven_zip_path: PathBuf,
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
            Key::Solo => Setting::Solo(self.solo),
        }
    }

    pub fn set(mut self, setting: Setting) -> Config {
        match setting {
            Setting::Dropbox(value) => {
                self.dropbox = value;
            }
            Setting::Saves(value) => {
                self.saves = value;
            }
            Setting::SevenZipPath(value) => {
                self.seven_zip_path = value;
            }
            Setting::Side(value) => {
                self.side = value;
            }
            Setting::Player(value) => {
                self.player = value;
            }
            Setting::Turn(value) => {
                self.turn = value;
            }
            Setting::Solo(value) => {
                self.solo = value;
            }
        }

        self
    }
}
