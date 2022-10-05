use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Side {
    Axis,
    Allies,
}

impl Side {
    /// guess whether a string contains either side, and return that side if it does
    pub(crate) fn detect_side_in_string(string: &str) -> Result<Self, UnknownSideError> {
        match string.to_lowercase() {
            s if s.contains("allies") => Ok(Side::Allies),
            s if s.contains("axis") => Ok(Side::Axis),
            _ => Err(UnknownSideError(string.to_string())),
        }
    }
}

impl Display for Side {
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
        Self::detect_side_in_string(s)
    }
}

#[derive(Debug, Error)]
#[error("Could not determine whether Axis or Allies")]
pub(crate) struct UnknownSideError(String);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn detect_sides() {
        let test_cases = [
            ("Allies 41", Side::Allies),
            ("Axis AB 12", Side::Axis),
            ("CD Allies 82", Side::Allies),
            ("a allies  fe123 ", Side::Allies),
            ("w 123 f axis a", Side::Axis),
        ];
        for tc in test_cases {
            let actual = Side::detect_side_in_string(tc.0).unwrap();
            assert_eq!(actual, tc.1);
        }
    }
}
