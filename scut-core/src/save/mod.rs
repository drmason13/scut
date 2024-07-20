mod autosave;
mod parse;
mod side;
mod turn;

use std::{cmp::Ordering, fmt, path::Path};

pub use self::parse::*;

pub use autosave::SaveOrAutosave;
use serde::{Deserialize, Serialize};
pub use side::Side;
pub use turn::Turn;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Save {
    pub player: Option<String>,
    pub turn: Turn,
    pub part: Option<String>,
}

impl Save {
    /// Create a new turn start Save for a given turn
    pub fn new(turn: Turn) -> Self {
        Save {
            turn,
            player: None,
            part: None,
        }
    }

    /// Create a turn start Save from the side and turn number
    pub fn from_parts(side: Side, turn: u32) -> Self {
        Save::new(Turn::new(side, turn))
    }

    /// Builder method to set the player of this save
    pub fn player<S>(mut self, player: S) -> Self
    where
        S: Into<String>,
    {
        self.player = Some(player.into());
        self
    }

    /// Builder method to set the part of this save
    pub fn part<S>(mut self, part: S) -> Self
    where
        S: Into<String>,
    {
        self.part = Some(part.into());
        self
    }

    /// Turn this save into the "turn start save" for next turn
    pub fn next_turn(self) -> Self {
        Save {
            player: None,
            turn: self.turn.next(),
            part: None,
        }
    }
}

impl fmt::Display for Save {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Save {
            turn: Turn { side, number: turn },
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

pub fn path_to_save(path: &Path) -> Option<Save> {
    Save::try_from(path).ok()
}

impl Ord for Save {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.turn.cmp(&other.turn) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match (&self.player, &other.player) {
            (Some(a), Some(b)) => match a.cmp(b) {
                Ordering::Equal => {}
                ord => return ord,
            },
            (None, Some(_)) => return Ordering::Greater,
            (Some(_), None) => return Ordering::Less,
            (None, None) => {}
        }
        match (&self.part, &other.part) {
            (Some(a), Some(b)) => a.cmp(b),
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }
}

impl PartialOrd for Save {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
