//! The [`Index`] interface allows retrieving [`Save`]s by turn, Side, player and/or part.
//!
//! [`storage`](crate::interface::storage) interfaces provide a compatible implementation of [`Index`] to allow searching with their store of saves via a method.

pub mod folder;

use crate::{Save, Side};

pub trait Index {
    fn search(&self, query: Query) -> Vec<Save>;
}

/// All the fields to filter Saves by, each field can be set to None to indicate a Wildcard search for that field
pub struct Query {
    pub turn: Option<u32>,
    pub side: Option<Side>,
    pub player: Option<String>,
    pub part: Option<String>,
}

impl Query {
    pub fn new() -> Self {
        Query {
            turn: None,
            side: None,
            player: None,
            part: None,
        }
    }

    pub fn turn(mut self, turn: u32) -> Self {
        self.turn = Some(turn);
        self
    }

    pub fn side(mut self, side: Side) -> Self {
        self.side = Some(side);
        self
    }

    pub fn player<S>(mut self, player: S) -> Self
    where
        S: Into<String>,
    {
        self.player = Some(player.into());
        self
    }

    pub fn part<S>(mut self, part: S) -> Self
    where
        S: Into<String>,
    {
        self.part = Some(part.into());
        self
    }
}

impl Default for Query {
    fn default() -> Self {
        Self::new()
    }
}
