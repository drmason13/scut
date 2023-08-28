use std::{fmt, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Key, Side};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Setting {
    Dropbox(PathBuf),
    Saves(PathBuf),
    SevenZipPath(PathBuf),
    Side(Side),
    Player(String),
    Turn(u32),
}

impl fmt::Display for Setting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    pub fn new(key: Key, value: String) -> anyhow::Result<Self> {
        match key {
            Key::Dropbox => Ok(Setting::Dropbox(value.into())),
            Key::Saves => Ok(Setting::Saves(value.into())),
            Key::SevenZipPath => Ok(Setting::SevenZipPath(value.into())),
            Key::Side => Ok(Setting::Side(value.parse()?)),
            Key::Player => Ok(Setting::Player(value)),
            Key::Turn => Ok(Setting::Turn(
                value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("invalid turn number"))?,
            )),
        }
    }
}
