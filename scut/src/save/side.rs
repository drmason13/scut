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
}

#[derive(Debug)]
pub struct UnknownSideError(String);

impl fmt::Display for UnknownSideError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown side {}", self.0)
    }
}

impl std::error::Error for UnknownSideError {}
