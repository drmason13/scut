use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Side {
    Axis,
    Allies,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Axis => write!(f, "Axis"),
            Self::Allies => write!(f, "Allies"),
        }
    }
}

impl FromStr for Side {
    type Err = UnknownSideError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase() {
            s if s.contains("allies") => Ok(Side::Allies),
            s if s.contains("axis") => Ok(Side::Axis),
            _ => Err(UnknownSideError(s.to_string())),
        }
    }
}

impl Side {
    pub fn other_side(&self) -> Self {
        match self {
            Self::Allies => Self::Axis,
            Self::Axis => Self::Allies,
        }
    }

    pub fn first() -> Self {
        Self::Axis
    }

    /// Return the next turn number for a given [`Side`]. Used when that [`Side`] is ending the turn.
    ///
    /// When Axis end the turn, the turn number stays the same.
    /// ```text
    /// Axis 1, Allies 1, Axis 2, Allies 2...
    /// ^^^^^^^^^^^^^^^^
    /// ```
    ///
    /// When Allies end the turn, the turn number increase by 1.
    /// ```text
    /// Axis 1, Allies 1, Axis 2, Allies 2...
    ///         ^^^^^^^^^^^^^^^^
    /// ```
    ///
    /// ```
    /// # use scut_core::Side;
    /// assert_eq!(Side::Axis.next_turn(7), 7);
    /// assert_eq!(Side::Allies.next_turn(7), 8);
    /// ```
    pub fn next_turn(&self, turn: u32) -> u32 {
        match self {
            Side::Allies => turn + 1,
            Side::Axis => turn,
        }
    }
}

#[derive(Debug)]
pub struct UnknownSideError(String);

impl fmt::Display for UnknownSideError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown side {}", self.0)
    }
}

impl std::error::Error for UnknownSideError {}
