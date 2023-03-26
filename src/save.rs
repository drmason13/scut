use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use yap::{
    // Allows you to use `.into_tokens()` on strings and slices,
    // to get an instance of the above:
    IntoTokens,
    // This trait has all of the parsing methods on it:
    Tokens,
};

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
        let mut tokens = s.into_tokens();

        parse_save(&mut tokens).ok_or(ParseSaveError)
    }
}

fn parse_save(tokens: &mut impl Tokens<Item = char>) -> Option<Save> {
    yap::one_of!(tokens;
        parse_autosave(tokens),
        parse_turnsave(tokens),
    )
}

fn parse_autosave(tokens: &mut impl Tokens<Item = char>) -> Option<Save> {
    yap::one_of!(tokens;
        tokens.tokens("autosave".chars()).then_some(Save::Autosave),
        tokens.tokens("Autosave".chars()).then_some(Save::Autosave),
    )
}

fn parse_turnsave(tokens: &mut impl Tokens<Item = char>) -> Option<Save> {
    yap::one_of!(tokens;
        // Side Start Turn
        tokens.optional(|tks| {
            let side = parse_side(tks)?;

            tks.optional(|ts| ws(ts));

            tks.optional(|ts| parse_start(ts));

            tks.optional(|ts| ws(ts));

            let turn = parse_turn(tks)?;

            let more = tks.next();

            if more.is_some() {
                return None
            }

            Some(Save::Turn(TurnSave {
                player: None,
                side,
                turn,
                part: None,
            }))
        }),

        // Side Player[ ]Turn
        tokens.optional(|tks| {
            let side = parse_side(tks)?;

            tks.optional(|ts| ws(ts));

            let player = parse_player(tks)?;

            tks.optional(|ts| ws(ts));

            let turn = parse_turn(tks)?;

            tks.optional(|ts| ws(ts));

            let part = tks.optional(|ts| parse_part(ts));

            Some(Save::Turn(TurnSave {
                player: Some(player),
                side,
                turn,
                part,
            }))
        }),

        // Side Turn[ ]Player
        tokens.optional(|tks| {
            let side = parse_side(tks)?;

            tks.optional(|ts| ws(ts));

            let turn = parse_turn(tks)?;

            tks.optional(|ts| ws(ts));

            let player = parse_player(tks)?;

            tks.optional(|ts| ws(ts));

            let part = tks.optional(|ts| parse_part(ts));

            Some(Save::Turn(TurnSave {
                player: Some(player),
                side,
                turn,
                part,
            }))
        }),
    )
}

fn ws(tokens: &mut impl Tokens<Item = char>) -> Option<()> {
    tokens.next()?.is_ascii_whitespace().then_some(())
}

fn parse_side(tokens: &mut impl Tokens<Item = char>) -> Option<Side> {
    yap::one_of!(tokens;
        tokens.tokens("Axis".chars()).then_some(Side::Axis),
        tokens.tokens("axis".chars()).then_some(Side::Axis),
        tokens.tokens("Allies".chars()).then_some(Side::Allies),
        tokens.tokens("allies".chars()).then_some(Side::Allies),
    )
}

fn parse_start(tokens: &mut impl Tokens<Item = char>) -> Option<()> {
    yap::one_of!(tokens;
        tokens.tokens("Start".chars()).then_some(()),
        tokens.tokens("start".chars()).then_some(()),
    )
}

fn parse_player(tokens: &mut impl Tokens<Item = char>) -> Option<String> {
    let matched: String = tokens.tokens_while(|tk| tk.is_alphabetic()).collect();

    (!matched.is_empty()).then_some(matched)
}

fn parse_turn(tokens: &mut impl Tokens<Item = char>) -> Option<u32> {
    let matched = tokens
        .tokens_while(|tk| tk.is_numeric())
        .collect::<String>();

    if matched.is_empty() {
        None
    } else {
        matched.parse::<u32>().ok()
    }
}

fn parse_part(tokens: &mut impl Tokens<Item = char>) -> Option<String> {
    tokens.optional(|tks| tks.tokens("part".chars()).then_some(()));
    tokens.optional(|tks| ws(tks));

    let matched: String = tokens
        .tokens_while(|tk| tk.is_alphanumeric() || tk.is_ascii_whitespace())
        .collect();
    if matched.is_empty() {
        None
    } else {
        Some(matched)
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
