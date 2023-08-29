//! The [`Index`] interface allows searching for [`Save`]s.
//!
//! [`storage`](crate::interface::storage) interfaces provide a compatible implementation of [`Index`]
//! to allow searching within their store of saves via the [`index`](crate::interface::LocalStorage::index) method.

pub mod folder;

use std::ops::{RangeFrom, RangeFull, RangeInclusive, RangeTo};

use crate::{Save, Side};

/// The [`Index`] interface allows searching for [`Save`]s by turn, Side, player and/or part using a query.
/// As well as getting the earliest or latest turn for a [`Side`].
pub trait Index {
    fn search(&self, query: Query) -> anyhow::Result<Vec<Save>>;

    /// Return the latest turn for a side, if it exists
    fn latest(&self, side: Side) -> anyhow::Result<Option<Save>>;

    /// Return the earliest turn for a side, if it exists
    fn earliest(&self, side: Side) -> anyhow::Result<Option<Save>>;
}

/// All the fields to filter Saves by, each field can be set to None to indicate a Wildcard search for that field
pub struct Query {
    pub turn: Option<TurnQuery>,
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

    /// Search for a particular turn
    pub fn turn(mut self, turn: u32) -> Self {
        self.turn = Some(TurnQuery::Single(turn));
        self
    }

    /// Search for a turn within a (possibly unbounded range). The range is inclusive, so the turn equal to `to` will match.
    ///
    /// Note that specifying a full range (where from and to are both None) is no different to not specifying a turn field at all
    pub fn turn_in_range(mut self, from: Option<u32>, to: Option<u32>) -> Self {
        self.turn = match (from, to) {
            (Some(a), Some(b)) => Some(TurnQuery::Inclusive(a..=b)),
            (Some(a), None) => Some(TurnQuery::LowerBounded(a..)),
            (None, Some(b)) => Some(TurnQuery::UpperBounded(..b)),
            (None, None) => None,
        };
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

/// All supported [`RangeBounds`](std::ops::RangeBounds) that [`Save`]s can be queried by with respect to their turn.
///
/// This is used in favor of generics so that the interface can remain object safe.
///
/// (I don't think the interfaces strictly *need* to be object safe but it might turn out to be helpful that they are)
pub enum TurnQuery {
    Single(u32),
    Inclusive(RangeInclusive<u32>),
    LowerBounded(RangeFrom<u32>),
    UpperBounded(RangeTo<u32>),
    Unbounded(RangeFull),
}
