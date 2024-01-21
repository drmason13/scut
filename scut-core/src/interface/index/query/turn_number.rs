use std::ops::{RangeFrom, RangeFull, RangeInclusive, RangeToInclusive};

use super::{Bool, Matches, Query, QueryParam};

impl From<u32> for TurnNumberRange {
    fn from(value: u32) -> Self {
        TurnNumberRange::single(value)
    }
}

/// All supported [`RangeBounds`](std::ops::RangeBounds) that [`Save`]s can be queried by with respect to their turn_number.
///
/// This is used in favor of generics so that the interface can remain object safe.
//
// (I don't think the interfaces strictly *need* to be object safe but it might turn out to be helpful that they are)
#[derive(Debug, Clone, PartialEq)]
pub enum TurnNumberRange {
    Single(u32),
    Inclusive(RangeInclusive<u32>),
    LowerBounded(RangeFrom<u32>),
    UpperBounded(RangeToInclusive<u32>),
    Unbounded(RangeFull),
}

impl TurnNumberRange {
    pub fn single(n: u32) -> Self {
        TurnNumberRange::Single(n)
    }

    pub fn from_start_end(start: Option<u32>, end: Option<u32>) -> Self {
        match (start, end) {
            (Some(a), Some(b)) => TurnNumberRange::Inclusive(a..=b),
            (Some(a), None) => TurnNumberRange::LowerBounded(a..),
            (None, Some(b)) => TurnNumberRange::UpperBounded(..=b),
            (None, None) => TurnNumberRange::Unbounded(..),
        }
    }
}

impl Matches<u32> for TurnNumberRange {
    fn matches(&self, value: &u32) -> bool {
        match &self {
            TurnNumberRange::Single(n) => n == value,
            TurnNumberRange::Inclusive(rng) => rng.contains(value),
            TurnNumberRange::LowerBounded(rng) => rng.contains(value),
            TurnNumberRange::UpperBounded(rng) => rng.contains(value),
            TurnNumberRange::Unbounded(_) => true,
        }
    }
}

impl Matches<u32> for Bool<TurnNumberRange> {
    fn matches(&self, value: &u32) -> bool {
        match self {
            Bool::Is(param) => param.matches(value),
            Bool::IsNot(param) => !param.matches(value),
        }
    }
}

impl Matches<u32> for QueryParam<TurnNumberRange> {
    fn matches(&self, value: &u32) -> bool {
        match self {
            QueryParam::Single(x) => x.matches(value),
            QueryParam::Multi(xs) => xs.iter().any(|x| x.matches(value)),
        }
    }
}

impl<'a> Query<'a> {
    /// Search for a particular turn number.
    ///
    /// Note this will overwrite any existing query param.
    pub fn turn_number(mut self, turn_number: u32) -> Self {
        let turn_number = QueryParam::is(TurnNumberRange::single(turn_number));
        self.turn_number = Some(turn_number);
        self
    }

    /// Add to a list of possible turn_numbers to match
    pub fn or_turn_number(mut self, turn_number: u32) -> Self {
        let turn_number = TurnNumberRange::single(turn_number);
        self.turn_number = self.turn_number.map(|n| n.or(turn_number));
        self
    }

    /// Search for any other turn number.
    ///
    /// Call `or_turn_number` after calling this method to build a list of turn_numbers to *not* match.
    ///
    /// # Examples
    ///
    /// NOT (A OR B)
    /// ```
    /// # use scut_core::interface::index::Query;
    /// # use compose::{Compose, Or, And};
    /// let query = Query::new()
    ///     .not_turn_number(1)
    ///     .or_turn_number(2);
    /// // matches turns 3, 4, 5... :)
    ///
    /// let query = Query::new()
    ///     .not_turn_number(1)
    ///     .or(Query::new().not_turn_number(2));
    /// // matches turns 1, 2, 3... :(
    /// ```
    ///
    /// (NOT A) OR B
    /// ```
    /// # use scut_core::interface::index::Query;
    /// # use compose::{Compose, Or, And};
    /// let query = Query::new()
    ///     .not_turn_number(1)
    ///     .or(Query::new().turn_number(2));
    /// ```
    ///
    pub fn not_turn_number(mut self, turn_number: u32) -> Self {
        let turn_number = QueryParam::not(TurnNumberRange::single(turn_number));
        self.turn_number = Some(turn_number);
        self
    }

    /// Add to a list of possible turn_numbers to not match
    pub fn or_not_turn_number(mut self, turn_number: u32) -> Self {
        let turn_number = TurnNumberRange::single(turn_number);
        self.turn_number = self.turn_number.map(|n| n.or_not(turn_number));
        self
    }

    /// Search for a turn number within a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will match.
    ///
    /// Note that specifying a full range (where from and to are both None) will match any turn_number
    pub fn turn_number_in_range(mut self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = QueryParam::is(TurnNumberRange::from_start_end(from, to));
        self.turn_number = Some(turn_number);
        self
    }

    /// Or search for a turn number within a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will match.
    ///
    /// Note that specifying a full range (where from and to are both None) will match any turn_number
    pub fn or_turn_number_in_range(mut self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = TurnNumberRange::from_start_end(from, to);
        self.turn_number = self.turn_number.map(|n| n.or(turn_number));
        self
    }

    /// Search for a turn number outside of a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will not match.
    ///
    /// Note that specifying a full range (where from and to are both None) cannot possibly match any turn_number
    pub fn turn_number_not_in_range(mut self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = QueryParam::not(TurnNumberRange::from_start_end(from, to));
        self.turn_number = Some(turn_number);
        self
    }

    /// Or search for a turn number within a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will match.
    ///
    /// Note that specifying a full range (where from and to are both None) will match any turn_number
    pub fn or_turn_number_not_in_range(mut self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = TurnNumberRange::from_start_end(from, to);
        self.turn_number = self.turn_number.map(|n| n.or_not(turn_number));
        self
    }
}
