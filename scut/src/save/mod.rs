mod parse;
mod side;

use std::{fmt, path::Path};

use self::parse::ParseSaveError;

pub use side::Side;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Save {
    pub player: Option<String>,
    pub side: Side,
    pub turn: u32,
    pub part: Option<String>,
}

impl Save {
    /// Turn this save into the "turn start save" for next turn
    pub fn next_turn(self) -> Self {
        let next_turn = match self.side {
            // Axis go first, so Allies play the same turn number next
            Side::Axis => self.turn,
            // Allies go last, so Axis play the next turn number next
            Side::Allies => self.turn + 1,
        };
        Save {
            player: None,
            side: self.side.other_side(),
            turn: next_turn,
            part: None,
        }
    }
}

impl fmt::Display for Save {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Save {
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
