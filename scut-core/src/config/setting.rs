use std::{fmt, path::PathBuf};

use parsely::{switch, token, Parse};
use serde::{Deserialize, Serialize};

use crate::{error::ErrorSuggestions, Key, Side};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Setting {
    Dropbox(PathBuf),
    Saves(PathBuf),
    SevenZipPath(PathBuf),
    Side(Side),
    Player(String),
    Turn(Option<u32>),
    Solo(Option<bool>),
}

impl fmt::Display for Setting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Setting::Dropbox(value) => value.display().fmt(f),
            Setting::Saves(value) => value.display().fmt(f),
            Setting::SevenZipPath(value) => value.display().fmt(f),
            Setting::Side(value) => value.fmt(f),
            Setting::Player(value) => value.fmt(f),
            Setting::Turn(value) => {
                if let Some(turn) = value {
                    turn.fmt(f)
                } else {
                    write!(f, "None")
                }
            }
            Setting::Solo(value) => value.unwrap_or_default().fmt(f),
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
            Key::Turn => Ok(Setting::Turn(Some(
                value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("`{value}` is not a valid turn number"))
                    .suggest(format!(
                        "Turn numbers must be whole numbers, max {}",
                        u32::MAX
                    ))?,
            ))),
            Key::Solo => Ok(Setting::Solo(Some({
                switch([
                    (token("true").any_case(), true),
                    (token("yes").any_case(), true),
                    (token("y").any_case(), true),
                    (token("1").any_case(), true),
                    (token("false").any_case(), false),
                    (token("no").any_case(), false),
                    (token("n").any_case(), false),
                    (token("0").any_case(), false),
                ])
                .parse(&value)
                .map_err(|_| anyhow::anyhow!("`{value}` is not a valid boolean"))
                .and_then(|(parsed, remaining)| {
                    if remaining.is_empty() {
                        Ok(parsed)
                    } else {
                        Err(anyhow::anyhow!("`{value}` is not a valid boolean"))
                    }
                })
                .suggest("config.solo should be set to 'true' or 'false'")?
            }))),
        }
    }
}
