use std::ops::{RangeFrom, RangeFull, RangeInclusive, RangeToInclusive};

use super::{Bool, LogicalCondition, Query};

/// All supported [`RangeBounds`](std::ops::RangeBounds) that [`Save`]s can be queried by with respect to their turn.
///
/// This is used in favor of generics so that the interface can remain object safe.
///
/// (I don't think the interfaces strictly *need* to be object safe but it might turn out to be helpful that they are)
#[derive(Debug, Clone, PartialEq)]
pub enum TurnQuery {
    Single(u32),
    Inclusive(RangeInclusive<u32>),
    LowerBounded(RangeFrom<u32>),
    UpperBounded(RangeToInclusive<u32>),
    Unbounded(RangeFull),
}

impl<'a> Query<'a> {
    /// Search for a particular turn.
    ///
    /// If this query is a [`CompoundQuery`], the first [`SubQuery`] is modified.
    ///
    /// [`CompoundQuery`]: Query::Compound
    pub fn turn(self, turn: u32) -> Self {
        match self {
            Query::Single(mut sub_query) => {
                sub_query.turn = Some(Bool::Is(TurnQuery::Single(turn)));
                Query::Single(sub_query)
            }
            Query::Compound(mut sub_query, op, q) => {
                sub_query.turn = Some(Bool::Is(TurnQuery::Single(turn)));
                Query::Compound(sub_query, op, q)
            }
        }
    }

    /// Or search for a particular turn.
    ///
    /// This will create a [`CompoundQuery`] using the [`Or`] condition, or update the second [`SubQuery`] and set the condition to [`Or`].
    ///
    /// Note that when creating a [`CompoundQuery`], the new [`SubQuery`] will only search by turn, none of the other [`SubQuery`]'s fields are copied.
    ///
    /// [`CompoundQuery`]: Query::Compound
    /// [`Or`]: LogicalCondition::Or
    pub fn or_turn(self, turn: u32) -> Self {
        match self {
            Query::Single(q) => match Query::new().turn(turn) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::Or, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.turn = Some(Bool::Is(TurnQuery::Single(turn)));
                Query::Compound(q, LogicalCondition::Or, sub_query)
            }
        }
    }

    /// And search for a particular turn.
    ///
    /// This will create a [`CompoundQuery`] using the [`And`] condition, or update the second [`SubQuery`] and set the condition to [`And`].
    ///
    /// Note that when creating a [`CompoundQuery`], the new [`SubQuery`] will only search by turn, none of the other [`SubQuery`]'s fields are copied.
    ///
    /// [`CompoundQuery`]: Query::Compound
    /// [`And`]: LogicalCondition::And
    pub fn and_turn(self, turn: u32) -> Self {
        match self {
            Query::Single(q) => match Query::new().turn(turn) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::And, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.turn = Some(Bool::Is(TurnQuery::Single(turn)));
                Query::Compound(q, LogicalCondition::And, sub_query)
            }
        }
    }

    /// Search for any other turn.
    ///
    /// If this query is a [`CompoundQuery`], the first [`SubQuery`] is modified.
    ///
    /// [`CompoundQuery`]: Query::Compound
    pub fn not_turn(self, turn: u32) -> Self {
        match self {
            Query::Single(mut sub_query) => {
                sub_query.turn = Some(Bool::IsNot(TurnQuery::Single(turn)));
                Query::Single(sub_query)
            }
            Query::Compound(mut sub_query, op, q) => {
                sub_query.turn = Some(Bool::IsNot(TurnQuery::Single(turn)));
                Query::Compound(sub_query, op, q)
            }
        }
    }

    /// Or search for any other turn.
    ///
    /// This will create a [`CompoundQuery`] using the [`Or`] condition, or update the second [`SubQuery`] and set the condition to [`Or`].
    ///
    /// Note that when creating a [`CompoundQuery`], the new [`SubQuery`] will only search by turn, none of the other [`SubQuery`]'s fields are copied.
    ///
    /// [`CompoundQuery`]: Query::Compound
    /// [`Or`]: LogicalCondition::Or
    pub fn or_not_turn(self, turn: u32) -> Self {
        match self {
            Query::Single(q) => match Query::new().not_turn(turn) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::Or, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(mut sub_query, _, q) => {
                sub_query.turn = Some(Bool::IsNot(TurnQuery::Single(turn)));
                Query::Compound(sub_query, LogicalCondition::Or, q)
            }
        }
    }

    /// And search for any other turn.
    ///
    /// This will create a [`CompoundQuery`] using the [`And`] condition, or update the second [`SubQuery`] and set the condition to [`And`].
    ///
    /// Note that when creating a [`CompoundQuery`], the new [`SubQuery`] will only search by turn, none of the other [`SubQuery`]'s fields are copied.
    ///
    /// [`CompoundQuery`]: Query::Compound
    /// [`And`]: LogicalCondition::And
    pub fn and_not_turn(self, turn: u32) -> Self {
        match self {
            Query::Single(q) => match Query::new().not_turn(turn) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::And, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(mut sub_query, op, q) => {
                sub_query.turn = Some(Bool::IsNot(TurnQuery::Single(turn)));
                Query::Compound(sub_query, op, q)
            }
        }
    }

    /// Search for a turn within a (possibly unbounded range). The range is inclusive, so the turn equal to `to` will match.
    ///
    /// Note that specifying a full range (where from and to are both None) will match any turn
    pub fn turn_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        match self {
            Query::Single(mut sub_query) => {
                sub_query.turn = match (from, to) {
                    (Some(a), Some(b)) => Some(Bool::Is(TurnQuery::Inclusive(a..=b))),
                    (Some(a), None) => Some(Bool::Is(TurnQuery::LowerBounded(a..))),
                    (None, Some(b)) => Some(Bool::Is(TurnQuery::UpperBounded(..=b))),
                    (None, None) => Some(Bool::Is(TurnQuery::Unbounded(..))),
                };
                Query::Single(sub_query)
            }
            Query::Compound(mut sub_query, op, q) => {
                sub_query.turn = match (from, to) {
                    (Some(a), Some(b)) => Some(Bool::Is(TurnQuery::Inclusive(a..=b))),
                    (Some(a), None) => Some(Bool::Is(TurnQuery::LowerBounded(a..))),
                    (None, Some(b)) => Some(Bool::Is(TurnQuery::UpperBounded(..=b))),
                    (None, None) => Some(Bool::Is(TurnQuery::Unbounded(..))),
                };
                Query::Compound(sub_query, op, q)
            }
        }
    }

    pub fn or_turn_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        match self {
            Query::Single(q) => match Query::new().turn_in_range(from, to) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::Or, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.turn = match (from, to) {
                    (Some(a), Some(b)) => Some(Bool::Is(TurnQuery::Inclusive(a..=b))),
                    (Some(a), None) => Some(Bool::Is(TurnQuery::LowerBounded(a..))),
                    (None, Some(b)) => Some(Bool::Is(TurnQuery::UpperBounded(..=b))),
                    (None, None) => Some(Bool::Is(TurnQuery::Unbounded(..))),
                };
                Query::Compound(q, LogicalCondition::Or, sub_query)
            }
        }
    }

    pub fn and_turn_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        match self {
            Query::Single(q) => match Query::new().turn_in_range(from, to) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::And, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.turn = match (from, to) {
                    (Some(a), Some(b)) => Some(Bool::Is(TurnQuery::Inclusive(a..=b))),
                    (Some(a), None) => Some(Bool::Is(TurnQuery::LowerBounded(a..))),
                    (None, Some(b)) => Some(Bool::Is(TurnQuery::UpperBounded(..=b))),
                    (None, None) => Some(Bool::Is(TurnQuery::Unbounded(..))),
                };
                Query::Compound(q, LogicalCondition::And, sub_query)
            }
        }
    }

    /// Search for a turn outside of a (possibly unbounded range). The range is inclusive, so the turn equal to `to` will not match.
    ///
    /// Note that specifying a full range (where from and to are both None) cannot possibly match any turn
    pub fn turn_not_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        match self {
            Query::Single(mut sub_query) => {
                sub_query.turn = match (from, to) {
                    (Some(a), Some(b)) => Some(Bool::IsNot(TurnQuery::Inclusive(a..=b))),
                    (Some(a), None) => Some(Bool::IsNot(TurnQuery::LowerBounded(a..))),
                    (None, Some(b)) => Some(Bool::IsNot(TurnQuery::UpperBounded(..=b))),
                    (None, None) => Some(Bool::IsNot(TurnQuery::Unbounded(..))),
                };
                Query::Single(sub_query)
            }
            Query::Compound(mut sub_query, op, q) => {
                sub_query.turn = match (from, to) {
                    (Some(a), Some(b)) => Some(Bool::IsNot(TurnQuery::Inclusive(a..=b))),
                    (Some(a), None) => Some(Bool::IsNot(TurnQuery::LowerBounded(a..))),
                    (None, Some(b)) => Some(Bool::IsNot(TurnQuery::UpperBounded(..=b))),
                    (None, None) => Some(Bool::IsNot(TurnQuery::Unbounded(..))),
                };
                Query::Compound(sub_query, op, q)
            }
        }
    }

    pub fn or_turn_not_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        match self {
            Query::Single(q) => match Query::new().turn_not_in_range(from, to) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::Or, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.turn = match (from, to) {
                    (Some(a), Some(b)) => Some(Bool::IsNot(TurnQuery::Inclusive(a..=b))),
                    (Some(a), None) => Some(Bool::IsNot(TurnQuery::LowerBounded(a..))),
                    (None, Some(b)) => Some(Bool::IsNot(TurnQuery::UpperBounded(..=b))),
                    (None, None) => Some(Bool::IsNot(TurnQuery::Unbounded(..))),
                };
                Query::Compound(q, LogicalCondition::Or, sub_query)
            }
        }
    }

    pub fn and_turn_not_in_range(self, from: Option<u32>, to: Option<u32>) -> Self {
        match self {
            Query::Single(q) => match Query::new().turn_not_in_range(from, to) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::And, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.turn = match (from, to) {
                    (Some(a), Some(b)) => Some(Bool::IsNot(TurnQuery::Inclusive(a..=b))),
                    (Some(a), None) => Some(Bool::IsNot(TurnQuery::LowerBounded(a..))),
                    (None, Some(b)) => Some(Bool::IsNot(TurnQuery::UpperBounded(..=b))),
                    (None, None) => Some(Bool::IsNot(TurnQuery::Unbounded(..))),
                };
                Query::Compound(q, LogicalCondition::And, sub_query)
            }
        }
    }
}
