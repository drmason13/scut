use std::ops::{RangeFrom, RangeFull, RangeInclusive, RangeToInclusive};

use super::{builder::QueryParam, Bool, Query, SubQuery};

/// Used to [`search`] for turn numbers in a range
///
/// [`search`]: crate::interface::Index::search
#[derive(Debug, Clone, PartialEq)]
pub struct TurnNumberQueryParam {
    boolean: Bool,
    range: TurnNumberRange,
}

impl TurnNumberQueryParam {
    pub fn single(boolean: Bool, n: u32) -> Self {
        TurnNumberQueryParam {
            boolean,
            range: TurnNumberRange::Single(n),
        }
    }

    pub fn from_start_end(boolean: Bool, start: Option<u32>, end: Option<u32>) -> Self {
        TurnNumberQueryParam {
            boolean,
            range: TurnNumberRange::from_start_end(start, end),
        }
    }
}

/// All supported [`RangeBounds`](std::ops::RangeBounds) that [`Save`]s can be queried by with respect to their turn_number.
///
/// This is used in favor of generics so that the interface can remain object safe.
///
/// (I don't think the interfaces strictly *need* to be object safe but it might turn out to be helpful that they are)
#[derive(Debug, Clone, PartialEq)]
pub enum TurnNumberRange {
    Single(u32),
    Inclusive(RangeInclusive<u32>),
    LowerBounded(RangeFrom<u32>),
    UpperBounded(RangeToInclusive<u32>),
    Unbounded(RangeFull),
}

impl TurnNumberRange {
    fn from_start_end(start: Option<u32>, end: Option<u32>) -> Self {
        match (start, end) {
            (Some(a), Some(b)) => TurnNumberRange::Inclusive(a..=b),
            (Some(a), None) => TurnNumberRange::LowerBounded(a..),
            (None, Some(b)) => TurnNumberRange::UpperBounded(..=b),
            (None, None) => TurnNumberRange::Unbounded(..),
        }
    }
}

impl<'a> QueryParam<'a> for TurnNumberQueryParam {
    type Value = u32;

    fn matches(&self, value: Self::Value) -> bool {
        todo!()
    }

    fn new_sub_query(self) -> super::SubQuery<'a> {
        SubQuery {
            turn_number: Some(self),
            ..Default::default()
        }
    }

    fn merge_into(self, mut sub_query: super::SubQuery<'a>) -> super::SubQuery<'a> {
        sub_query.turn_number = Some(self);
        sub_query
    }
}

impl<'a> Query<'a> {
    /// Search for a particular turn number.
    ///
    /// If this query is a [`CompoundQuery`], the first [`SubQuery`] is modified.
    ///
    /// [`CompoundQuery`]: Query::Compound
    pub fn turn_number(self, turn_number: u32) -> Self {
        let turn_number = TurnNumberQueryParam::single(Bool::Is, turn_number);
        turn_number.apply(self)
    }

    /// Or search for a particular turn number.
    ///
    /// This will create a [`CompoundQuery`] using the [`Or`] condition, or update the second [`SubQuery`] and set the condition to [`Or`].
    ///
    /// Note that when creating a [`CompoundQuery`], the new [`SubQuery`] will only search by turn_number, none of the other [`SubQuery`]'s fields are copied.
    ///
    /// [`CompoundQuery`]: Query::Compound
    /// [`Or`]: crate::interface::index::query::LogicalCondition::Or
    pub fn or_turn_number(self, turn_number: u32) -> Self {
        let turn_number = TurnNumberQueryParam::single(Bool::Is, turn_number);
        turn_number.apply_or(self)
    }

    /// And search for a particular turn number.
    ///
    /// This will create a [`CompoundQuery`] using the [`And`] condition, or update the second [`SubQuery`] and set the condition to [`And`].
    ///
    /// Note that when creating a [`CompoundQuery`], the new [`SubQuery`] will only search by turn_number, none of the other [`SubQuery`]'s fields are copied.
    ///
    /// [`CompoundQuery`]: Query::Compound
    /// [`And`]: crate::interface::index::query::LogicalCondition::And
    pub fn and_turn_number(self, turn_number: u32) -> Self {
        let turn_number = TurnNumberQueryParam::single(Bool::Is, turn_number);
        turn_number.apply_and(self)
    }

    /// Search for any other turn number.
    ///
    /// If this query is a [`CompoundQuery`], the first [`SubQuery`] is modified.
    ///
    /// [`CompoundQuery`]: Query::Compound
    pub fn not_turn_number(self, turn_number: u32) -> Self {
        let turn_number = TurnNumberQueryParam::single(Bool::Not, turn_number);
        turn_number.apply(self)
    }

    /// Or search for any other turn number.
    ///
    /// This will create a [`CompoundQuery`] using the [`Or`] condition, or update the second [`SubQuery`] and set the condition to [`Or`].
    ///
    /// Note that when creating a [`CompoundQuery`], the new [`SubQuery`] will only search by turn_number, none of the other [`SubQuery`]'s fields are copied.
    ///
    /// [`CompoundQuery`]: Query::Compound
    /// [`Or`]: crate::interface::index::query::LogicalCondition::Or
    pub fn or_not_turn_number(self, turn_number: u32) -> Self {
        let turn_number = TurnNumberQueryParam::single(Bool::Not, turn_number);
        turn_number.apply_or(self)
    }

    /// And search for any other turn number.
    ///
    /// This will create a [`CompoundQuery`] using the [`And`] condition, or update the second [`SubQuery`] and set the condition to [`And`].
    ///
    /// Note that when creating a [`CompoundQuery`], the new [`SubQuery`] will only search by turn_number, none of the other [`SubQuery`]'s fields are copied.
    ///
    /// [`CompoundQuery`]: Query::Compound
    /// [`And`]: crate::interface::index::query::LogicalCondition::And
    pub fn and_not_turn_number(self, turn_number: u32) -> Self {
        let turn_number = TurnNumberQueryParam::single(Bool::Not, turn_number);
        turn_number.apply_and(self)
    }

    /// Search for a turn number within a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will match.
    ///
    /// Note that specifying a full range (where from and to are both None) will match any turn_number
    pub fn turn_number_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = TurnNumberQueryParam::from_start_end(Bool::Is, from, to);
        turn_number.apply(self)
    }

    /// Or search for a turn number within a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will match.
    ///
    /// Note that specifying a full range (where from and to are both None) will match any turn_number
    pub fn or_turn_number_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = TurnNumberQueryParam::from_start_end(Bool::Is, from, to);
        turn_number.apply_or(self)
    }

    /// And search for a turn number within a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will match.
    ///
    /// Note that specifying a full range (where from and to are both None) will match any turn_number
    pub fn and_turn_number_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = TurnNumberQueryParam::from_start_end(Bool::Is, from, to);
        turn_number.apply_and(self)
    }

    /// Search for a turn number outside of a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will not match.
    ///
    /// Note that specifying a full range (where from and to are both None) cannot possibly match any turn_number
    pub fn turn_number_not_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = TurnNumberQueryParam::from_start_end(Bool::Not, from, to);
        turn_number.apply_and(self)
    }

    /// Or search for a turn number outside of a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will not match.
    ///
    /// Note that specifying a full range (where from and to are both None) cannot possibly match any turn_number
    pub fn or_turn_number_not_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = TurnNumberQueryParam::from_start_end(Bool::Not, from, to);
        turn_number.apply_or(self)
    }

    /// And search for a turn number outside of a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will not match.
    ///
    /// Note that specifying a full range (where from and to are both None) cannot possibly match any turn_number
    pub fn and_turn_number_not_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = TurnNumberQueryParam::from_start_end(Bool::Not, from, to);
        turn_number.apply_and(self)
    }
}
