use std::ops::{RangeFrom, RangeFull, RangeInclusive, RangeToInclusive};

use super::{builder::QueryBuildParameter, Bool, Query, SubQuery};

/// All supported [`RangeBounds`](std::ops::RangeBounds) that [`Save`]s can be queried by with respect to their turn_number.
///
/// This is used in favor of generics so that the interface can remain object safe.
///
/// (I don't think the interfaces strictly *need* to be object safe but it might turn out to be helpful that they are)
#[derive(Debug, Clone, PartialEq)]
pub enum TurnNumberQuery {
    Single(u32),
    Inclusive(RangeInclusive<u32>),
    LowerBounded(RangeFrom<u32>),
    UpperBounded(RangeToInclusive<u32>),
    Unbounded(RangeFull),
}

impl<'a> QueryBuildParameter<'a> for TurnNumberQuery {
    fn new_sub_query(self, boolean: bool) -> super::SubQuery<'a> {
        SubQuery {
            turn_number: Some(if boolean {
                Bool::Is(self)
            } else {
                Bool::IsNot(self)
            }),
            ..Default::default()
        }
    }

    fn merge_into(self, mut sub_query: super::SubQuery<'a>, boolean: bool) -> super::SubQuery<'a> {
        sub_query.turn_number = Some(if boolean {
            Bool::Is(self)
        } else {
            Bool::IsNot(self)
        });
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
        let turn_number = TurnNumberQuery::Single(turn_number);
        turn_number.build(self)
    }

    /// Or search for a particular turn number.
    ///
    /// This will create a [`CompoundQuery`] using the [`Or`] condition, or update the second [`SubQuery`] and set the condition to [`Or`].
    ///
    /// Note that when creating a [`CompoundQuery`], the new [`SubQuery`] will only search by turn_number, none of the other [`SubQuery`]'s fields are copied.
    ///
    /// [`CompoundQuery`]: Query::Compound
    /// [`Or`]: LogicalCondition::Or
    pub fn or_turn_number(self, turn_number: u32) -> Self {
        let turn_number = TurnNumberQuery::Single(turn_number);
        turn_number.build_or(self)
    }

    /// And search for a particular turn number.
    ///
    /// This will create a [`CompoundQuery`] using the [`And`] condition, or update the second [`SubQuery`] and set the condition to [`And`].
    ///
    /// Note that when creating a [`CompoundQuery`], the new [`SubQuery`] will only search by turn_number, none of the other [`SubQuery`]'s fields are copied.
    ///
    /// [`CompoundQuery`]: Query::Compound
    /// [`And`]: LogicalCondition::And
    pub fn and_turn_number(self, turn_number: u32) -> Self {
        let turn_number = TurnNumberQuery::Single(turn_number);
        turn_number.build_and(self)
    }

    /// Search for any other turn number.
    ///
    /// If this query is a [`CompoundQuery`], the first [`SubQuery`] is modified.
    ///
    /// [`CompoundQuery`]: Query::Compound
    pub fn not_turn_number(self, turn_number: u32) -> Self {
        let turn_number = TurnNumberQuery::Single(turn_number);
        turn_number.build_not(self)
    }

    /// Or search for any other turn number.
    ///
    /// This will create a [`CompoundQuery`] using the [`Or`] condition, or update the second [`SubQuery`] and set the condition to [`Or`].
    ///
    /// Note that when creating a [`CompoundQuery`], the new [`SubQuery`] will only search by turn_number, none of the other [`SubQuery`]'s fields are copied.
    ///
    /// [`CompoundQuery`]: Query::Compound
    /// [`Or`]: LogicalCondition::Or
    pub fn or_not_turn_number(self, turn_number: u32) -> Self {
        let turn_number = TurnNumberQuery::Single(turn_number);
        turn_number.build_or_not(self)
    }

    /// And search for any other turn number.
    ///
    /// This will create a [`CompoundQuery`] using the [`And`] condition, or update the second [`SubQuery`] and set the condition to [`And`].
    ///
    /// Note that when creating a [`CompoundQuery`], the new [`SubQuery`] will only search by turn_number, none of the other [`SubQuery`]'s fields are copied.
    ///
    /// [`CompoundQuery`]: Query::Compound
    /// [`And`]: LogicalCondition::And
    pub fn and_not_turn_number(self, turn_number: u32) -> Self {
        let turn_number = TurnNumberQuery::Single(turn_number);
        turn_number.build_and_not(self)
    }

    /// Search for a turn number within a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will match.
    ///
    /// Note that specifying a full range (where from and to are both None) will match any turn_number
    pub fn turn_number_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = match (from, to) {
            (Some(a), Some(b)) => TurnNumberQuery::Inclusive(a..=b),
            (Some(a), None) => TurnNumberQuery::LowerBounded(a..),
            (None, Some(b)) => TurnNumberQuery::UpperBounded(..=b),
            (None, None) => TurnNumberQuery::Unbounded(..),
        };
        turn_number.build(self)
    }

    /// Or search for a turn number within a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will match.
    ///
    /// Note that specifying a full range (where from and to are both None) will match any turn_number
    pub fn or_turn_number_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = match (from, to) {
            (Some(a), Some(b)) => TurnNumberQuery::Inclusive(a..=b),
            (Some(a), None) => TurnNumberQuery::LowerBounded(a..),
            (None, Some(b)) => TurnNumberQuery::UpperBounded(..=b),
            (None, None) => TurnNumberQuery::Unbounded(..),
        };
        turn_number.build_or(self)
    }

    /// And search for a turn number within a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will match.
    ///
    /// Note that specifying a full range (where from and to are both None) will match any turn_number
    pub fn and_turn_number_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = match (from, to) {
            (Some(a), Some(b)) => TurnNumberQuery::Inclusive(a..=b),
            (Some(a), None) => TurnNumberQuery::LowerBounded(a..),
            (None, Some(b)) => TurnNumberQuery::UpperBounded(..=b),
            (None, None) => TurnNumberQuery::Unbounded(..),
        };
        turn_number.build_and(self)
    }

    /// Search for a turn number outside of a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will not match.
    ///
    /// Note that specifying a full range (where from and to are both None) cannot possibly match any turn_number
    pub fn turn_number_not_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = match (from, to) {
            (Some(a), Some(b)) => TurnNumberQuery::Inclusive(a..=b),
            (Some(a), None) => TurnNumberQuery::LowerBounded(a..),
            (None, Some(b)) => TurnNumberQuery::UpperBounded(..=b),
            (None, None) => TurnNumberQuery::Unbounded(..),
        };
        turn_number.build_and_not(self)
    }

    /// Or search for a turn number outside of a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will not match.
    ///
    /// Note that specifying a full range (where from and to are both None) cannot possibly match any turn_number
    pub fn or_turn_number_not_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = match (from, to) {
            (Some(a), Some(b)) => TurnNumberQuery::Inclusive(a..=b),
            (Some(a), None) => TurnNumberQuery::LowerBounded(a..),
            (None, Some(b)) => TurnNumberQuery::UpperBounded(..=b),
            (None, None) => TurnNumberQuery::Unbounded(..),
        };
        turn_number.build_or_not(self)
    }

    /// And search for a turn number outside of a (possibly unbounded range). The range is inclusive, so the turn_number equal to `to` will not match.
    ///
    /// Note that specifying a full range (where from and to are both None) cannot possibly match any turn_number
    pub fn and_turn_number_not_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        let turn_number = match (from, to) {
            (Some(a), Some(b)) => TurnNumberQuery::Inclusive(a..=b),
            (Some(a), None) => TurnNumberQuery::LowerBounded(a..),
            (None, Some(b)) => TurnNumberQuery::UpperBounded(..=b),
            (None, None) => TurnNumberQuery::Unbounded(..),
        };
        turn_number.build_and_not(self)
    }
}
