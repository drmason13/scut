use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use parsely::{alpha, alphanum, char, digit, token, Lex, Parse, ParseResult};

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
    pub(crate) part: Option<String>,
}

/// Type wrapper around the two file formats and their extension
pub(crate) enum SavOrArchive {
    Sav,
    Archive,
}

impl SavOrArchive {
    pub(crate) fn extension(&self) -> &'static str {
        match self {
            Self::Sav => "sav",
            Self::Archive => "7z",
        }
    }
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
            part: None,
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
            part: None,
        }
    }
}

impl Display for TurnSave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let TurnSave {
            side,
            turn,
            player,
            part,
        } = self;
        match (player, part) {
            (Some(ref player), Some(ref part)) => write!(f, "{side} {player} {turn}{part}"),
            (Some(ref player), None) => write!(f, "{side} {player} {turn}"),
            _ => write!(f, "{side} {turn}"),
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
        parse_save(s).ok_or(ParseSaveError)
    }
}

fn parse_save(input: &str) -> Option<Save> {
    let (save, _) = parse_autosave.or(parse_turnsave).parse(input).ok()?;

    Some(save)
}

fn parse_autosave(input: &str) -> ParseResult<'_, Save> {
    token("autosave")
        .or(token("Autosave"))
        .map(|_| Save::Autosave)
        .parse(input)
}

fn parse_side(input: &str) -> ParseResult<'_, Side> {
    token("Allies")
        .or(token("allies"))
        .map(|_| Side::Allies)
        .or(token("Axis").or(token("axis")).map(|_| Side::Axis))
        .parse(input)
}

fn parse_player(input: &str) -> ParseResult<'_, String> {
    alpha().many(1..100).map(|s| s.to_string()).parse(input)
}

fn parse_turn(input: &str) -> ParseResult<'_, u32> {
    digit()
        .many(1..5)
        .try_map(|n| n.parse::<u32>())
        .parse(input)
}

fn parse_part(input: &str) -> ParseResult<'_, String> {
    let (_, remaining) = token("part ").optional().lex(input)?;
    alphanum()
        .many(1..100)
        .map(|s| s.to_string())
        .parse(remaining)
}

fn parse_turnsave(input: &str) -> ParseResult<'_, Save> {
    // Side Start Turn
    // "Axis[ ]start 123";
    let side_start_turn = parse_side
        .then_skip(char(' ').optional())
        .then_skip(token("start").any_case())
        .then_skip(char(' ').optional())
        .then(parse_turn)
        .mapped(|(side, turn)| {
            Save::Turn(TurnSave {
                player: None,
                side,
                turn,
                part: None,
            })
        });

    // Side[ ]Player[ ]Turn
    // "Axis DM 123";
    let side_player_turn = parse_side
        .then_skip(char(' ').optional())
        .then(parse_player)
        .then_skip(char(' ').optional())
        .then(parse_turn)
        .then_skip(char(' ').optional())
        .then(parse_part.optional())
        .mapped(|(((side, player), turn), part)| {
            Save::Turn(TurnSave {
                player: Some(player),
                side,
                turn,
                part,
            })
        });

    // Side[ ]Turn[ ]Player
    // "Axis 123 DM";
    let side_turn_player = parse_side
        .then_skip(char(' ').optional())
        .then(parse_turn)
        .then_skip(char(' ').optional())
        .then(parse_player)
        .then_skip(char(' ').optional())
        .then(parse_part.optional())
        .mapped(|(((side, turn), player), part)| {
            Save::Turn(TurnSave {
                player: Some(player),
                side,
                turn,
                part,
            })
        });

    // Side[ ]Turn
    // "Axis 123";
    let side_turn = parse_side
        .then_skip(char(' ').optional())
        .then(parse_turn)
        .mapped(|(side, turn)| {
            Save::Turn(TurnSave {
                player: None,
                side,
                turn,
                part: None,
            })
        });

    (side_start_turn)
        .or(side_turn_player)
        .or(side_player_turn)
        .or(side_turn)
        .parse(input)
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
            part: None,
        });
        assert_eq!(actual, expected);

        let save = "Allies DM 123";
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("DM".into()),
            turn: 123,
            side: Side::Allies,
            part: None,
        });
        assert_eq!(actual, expected);

        let save = "Axis 123 DM";
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("DM".into()),
            turn: 123,
            side: Side::Axis,
            part: None,
        });
        assert_eq!(actual, expected);

        let save = "Axis Start 123";
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: None,
            turn: 123,
            side: Side::Axis,
            part: None,
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
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: None,
            turn: 123,
            side: Side::Allies,
            part: None,
        });
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
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("dm".into()),
            turn: 123,
            side: Side::Axis,
            part: None,
        });
        assert_eq!(actual, expected);

        let save = "axis123 dm";
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("dm".into()),
            turn: 123,
            side: Side::Axis,
            part: None,
        });
        assert_eq!(actual, expected);

        let save = "Allies 123 DM part A";
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("DM".into()),
            turn: 123,
            side: Side::Allies,
            part: Some("A".into()),
        });
        assert_eq!(actual, expected);

        let save = "Allies 123 DMB";
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("DMB".into()),
            turn: 123,
            side: Side::Allies,
            part: None,
        });
        assert_eq!(actual, expected);

        let save = "Allies DM 123C";
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("DM".into()),
            turn: 123,
            side: Side::Allies,
            part: Some("C".into()),
        });
        assert_eq!(actual, expected);

        let save = "Allies DM 123 D";
        let actual: Save = save.parse().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("DM".into()),
            turn: 123,
            side: Side::Allies,
            part: Some("D".into()),
        });
        assert_eq!(actual, expected);

        let save = "Allies DM Part 123 D";
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
            part: None,
        });
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/Allies DM 123.sav");
        let actual: Save = save.try_into().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("DM".into()),
            turn: 123,
            side: Side::Allies,
            part: None,
        });
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/Axis 123 DM.7z");
        let actual: Save = save.try_into().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("DM".into()),
            turn: 123,
            side: Side::Axis,
            part: None,
        });
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/Axis Start 123.7z");
        let actual: Save = save.try_into().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: None,
            turn: 123,
            side: Side::Axis,
            part: None,
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
        let actual: Save = save.try_into().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: None,
            turn: 123,
            side: Side::Allies,
            part: None,
        });
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
        let actual: Save = save.try_into().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("dm".into()),
            turn: 123,
            side: Side::Axis,
            part: None,
        });
        assert_eq!(actual, expected);

        let save = PathBuf::from("foo/bar/axis123 dm.sav");
        let actual: Save = save.try_into().expect("should parse");
        let expected = Save::Turn(TurnSave {
            player: Some("dm".into()),
            turn: 123,
            side: Side::Axis,
            part: None,
        });
        assert_eq!(actual, expected);
    }
}
