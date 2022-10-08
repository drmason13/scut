use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use either::Either;
use thiserror::Error;

use crate::{config::Config, side::Side};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Save {
    Autosave,
    Turn(TurnSave),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TurnSave {
    pub(crate) player: Option<String>,
    pub(crate) side: Side,
    pub(crate) turn: u32,
}

impl Save {
    pub(crate) fn is_autosave(&self) -> bool {
        matches!(self, Save::Autosave)
    }
}

impl Display for Save {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Save::Autosave => write!(f, "autosave"),
            Save::Turn(save) => save.fmt(f),
        }
    }
}

impl TurnSave {
    pub(crate) fn from_config(config: &Config) -> Self {
        TurnSave {
            player: Some(config.player.clone()),
            side: config.side,
            turn: config.turn,
        }
    }

    pub(crate) fn next_turn(self) -> Self {
        let next_turn = match self.side {
            // Axis go first, so Allies play the same turn number next
            Side::Axis => self.turn,
            // Allies go last, so Axis play the next turn number next
            Side::Allies => self.turn + 1,
        };
        TurnSave {
            player: None,
            side: self.side.other_side(),
            turn: next_turn,
        }
    }
}

impl Display for TurnSave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let TurnSave { side, turn, player } = self;
        if let Some(ref player) = player {
            write!(f, "{side} {player} {turn}")
        } else {
            write!(f, "{side} {turn}")
        }
    }
}

impl TryFrom<PathBuf> for Save {
    type Error = ParseSaveError;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        value
            .file_name()
            .ok_or(ParseSaveError)?
            .to_string_lossy()
            .split('.')
            .next()
            .ok_or(ParseSaveError)?
            .parse()
    }
}

impl TryFrom<&Path> for Save {
    type Error = ParseSaveError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        value
            .file_name()
            .ok_or(ParseSaveError)?
            .to_string_lossy()
            .split('.')
            .next()
            .ok_or(ParseSaveError)?
            .parse()
    }
}

impl TryFrom<&PathBuf> for Save {
    type Error = ParseSaveError;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        value
            .file_name()
            .ok_or(ParseSaveError)?
            .to_string_lossy()
            .split('.')
            .next()
            .ok_or(ParseSaveError)?
            .parse()
    }
}

impl FromStr for Save {
    type Err = ParseSaveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "autosave" {
            return Ok(Save::Autosave);
        }

        let mut terms = s.split(' ');

        let side: Side = terms
            .next()
            .ok_or(ParseSaveError)?
            .parse()
            .map_err(|_| ParseSaveError)?;

        let turn_or_player: Either<u32, Option<String>> = match terms.next() {
            Some(term) if term.parse::<u32>().is_ok() => Either::Left(term.parse().unwrap()),
            Some(term) if term.to_lowercase() != "start" => Either::Right(Some(term.to_string())),
            Some(_) => Either::Right(None),
            None => return Err(ParseSaveError),
        };

        let (turn, player) = match turn_or_player {
            Either::Right(player) => (
                {
                    // turn
                    match terms.next() {
                        Some(term) => term.parse::<u32>().map_err(|_| ParseSaveError)?,
                        None => return Err(ParseSaveError),
                    }
                },
                player,
            ),
            Either::Left(turn) => (turn, {
                // player
                match terms.next() {
                    Some(term) if term.to_lowercase() != "start" => Some(term.to_string()),
                    Some(_) => None,
                    None => None,
                }
            }),
        };

        Ok(Save::Turn(TurnSave { player, side, turn }))
    }
}

#[derive(Debug, Error, PartialEq)]
#[error("Could not parse save filename")]
pub(crate) struct ParseSaveError;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_save() {
        let save = "Allies 123";
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: None,
            turn: 123,
            side: Side::Allies,
        });
        assert_eq!(actual, expected);

        let save = "Allies DM 123";
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("DM".into()),
            turn: 123,
            side: Side::Allies,
        });
        assert_eq!(actual, expected);

        let save = "Axis 123 DM";
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("DM".into()),
            turn: 123,
            side: Side::Axis,
        });
        assert_eq!(actual, expected);

        let save = "Axis Start 123";
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: None,
            turn: 123,
            side: Side::Axis,
        });
        assert_eq!(actual, expected);

        let save = "autosave";
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Autosave;
        assert_eq!(actual, expected);

        let save = "axis";
        let actual: Result<Save, ParseSaveError> = save.parse();
        let expected = Err(ParseSaveError);
        assert_eq!(actual, expected);

        let save = "allies123";
        let actual: Result<Save, ParseSaveError> = save.parse();
        let expected = Err(ParseSaveError);
        assert_eq!(actual, expected);

        let save = "123 axis dm";
        let actual: Result<Save, ParseSaveError> = save.parse();
        let expected = Err(ParseSaveError);
        assert_eq!(actual, expected);

        let save = "dm 123 axis";
        let actual: Result<Save, ParseSaveError> = save.parse();
        let expected = Err(ParseSaveError);
        assert_eq!(actual, expected);

        let save = "dm axis 123";
        let actual: Result<Save, ParseSaveError> = save.parse();
        let expected = Err(ParseSaveError);
        assert_eq!(actual, expected);

        let save = "axis 123dm";
        let actual: Result<Save, ParseSaveError> = save.parse();
        let expected = Err(ParseSaveError);
        assert_eq!(actual, expected);

        let save = "axis123 dm";
        let actual: Result<Save, ParseSaveError> = save.parse();
        let expected = Err(ParseSaveError);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_save_path() {
        let save = PathBuf::from("foo/bar/Allies 123.7z");
        let actual: Save = save.try_into().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: None,
            turn: 123,
            side: Side::Allies,
        });
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/Allies DM 123.sav");
        let actual: Save = save.try_into().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("DM".into()),
            turn: 123,
            side: Side::Allies,
        });
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/Axis 123 DM.7z");
        let actual: Save = save.try_into().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("DM".into()),
            turn: 123,
            side: Side::Axis,
        });
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/Axis Start 123.7z");
        let actual: Save = save.try_into().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: None,
            turn: 123,
            side: Side::Axis,
        });
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/autosave.sav");
        let actual: Save = save.try_into().expect("should parse");
        let expected = Save::Autosave;
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/axis.sav");
        let actual: Result<Save, ParseSaveError> = save.try_into();
        let expected = Err(ParseSaveError);
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/allies123.sav");
        let actual: Result<Save, ParseSaveError> = save.try_into();
        let expected = Err(ParseSaveError);
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/123 axis dm.sav");
        let actual: Result<Save, ParseSaveError> = save.try_into();
        let expected = Err(ParseSaveError);
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/dm 123 axis.sav");
        let actual: Result<Save, ParseSaveError> = save.try_into();
        let expected = Err(ParseSaveError);
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/dm axis 123.sav");
        let actual: Result<Save, ParseSaveError> = save.try_into();
        let expected = Err(ParseSaveError);
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/axis 123dm.sav");
        let actual: Result<Save, ParseSaveError> = save.try_into();
        let expected = Err(ParseSaveError);
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/axis123 dm.sav");
        let actual: Result<Save, ParseSaveError> = save.try_into();
        let expected = Err(ParseSaveError);
        assert_eq!(actual, expected);
    }
}
