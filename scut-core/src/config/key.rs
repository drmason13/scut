use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Key {
    Dropbox,
    Saves,
    SevenZipPath,
    Side,
    Player,
    Turn,
}

impl fmt::Display for Key {
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
    type Err = anyhow::Error;

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
            key => anyhow::bail!("invalid key: {key}"),
        }
    }
}
