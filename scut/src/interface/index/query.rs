use std::ops::{RangeFrom, RangeFull, RangeInclusive, RangeToInclusive};

use crate::{Save, Side};

/// A "full blown" [`Query`] contains 1 or 2 [`NestedQuery`]s.
/// The [`NestedQuery`] contains a logical condition ([`And`] or [`Or`]) which describes its relation to its parent.
///
/// Each [`NestedQuery`] is either a "leaf" [`SubQuery`] which contains a full set of conditions, or another [`Query`] which
/// itself contains further [`SubQuery`]s. Any such [`Query`]s have to be boxed to avoid an infinitely sized type.
///
/// This forms a tree structure:
///
/// ```
/// Query::Compound(Query::Single(A), And, Query::Single(B))
/// // implies "A AND B"
///
/// // whereas "A OR B" would be
/// Query::Compound(Query::Single(A), Or, Query::Single(B))
///
/// // And finally just C is
/// Query::Single(C)
/// ```
///
/// More complex nested compound queries could be supported, but the lifetimes become awkward and Boxes become required and it's not required.
#[derive(Debug, Clone, PartialEq)]
pub enum Query<'a> {
    Single(SubQuery<'a>),
    Compound(SubQuery<'a>, LogicalCondition, SubQuery<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalCondition {
    And,
    Or,
}

/// All the fields to filter [`Save`]s by, each field can be set to `None` to indicate a Wildcard search for that field
#[derive(Debug, Clone, PartialEq)]
pub struct SubQuery<'a> {
    /// match the [`TurnQuery`] (either a single turn or within a range)
    pub turn: Option<Bool<TurnQuery>>,
    /// match the side
    pub side: Option<Bool<Side>>,
    /// match the player - use `Some(None)` to match saves with no player (i.e. turn start saves)
    pub player: Option<Bool<Option<&'a str>>>,
    /// match the part - use `Some(None)` to match saves with no part
    pub part: Option<Bool<Option<&'a str>>>,
}

impl<'a> Query<'a> {
    pub fn new() -> Self {
        Query::Single(SubQuery {
            turn: None,
            side: None,
            player: None,
            part: None,
        })
    }

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

    pub fn or_turn_in_range(self, _from: Option<u32>, _to: Option<u32>) -> Self {
        todo!()
    }

    pub fn and_turn_in_range(self, _from: Option<u32>, _to: Option<u32>) -> Self {
        todo!()
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

    pub fn or_turn_not_in_range(self, _from: Option<u32>, _to: Option<u32>) -> Self {
        todo!()
    }

    pub fn and_turn_not_in_range(self, _from: Option<u32>, _to: Option<u32>) -> Self {
        todo!()
    }

    /// Search for a particular side
    pub fn side(self, side: Side) -> Self {
        match self {
            Query::Single(mut sub_query) => {
                sub_query.side = Some(Bool::Is(side));
                Query::Single(sub_query)
            }
            Query::Compound(mut sub_query, op, q) => {
                sub_query.side = Some(Bool::Is(side));
                Query::Compound(sub_query, op, q)
            }
        }
    }

    /// Or search for a particular side
    pub fn or_side(self, side: Side) -> Self {
        match self {
            Query::Single(q) => match Query::new().side(side) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::Or, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.side = Some(Bool::Is(side));
                Query::Compound(q, LogicalCondition::Or, sub_query)
            }
        }
    }

    /// And search for a particular side
    pub fn and_side(self, side: Side) -> Self {
        match self {
            Query::Single(q) => match Query::new().side(side) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::And, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.side = Some(Bool::Is(side));
                Query::Compound(q, LogicalCondition::And, sub_query)
            }
        }
    }

    /// Search for any other side
    pub fn not_side(self, side: Side) -> Self {
        match self {
            Query::Single(mut sub_query) => {
                sub_query.side = Some(Bool::IsNot(side));
                Query::Single(sub_query)
            }
            Query::Compound(mut sub_query, op, q) => {
                sub_query.side = Some(Bool::IsNot(side));
                Query::Compound(sub_query, op, q)
            }
        }
    }

    /// Or search for any other side
    pub fn or_not_side(self, side: Side) -> Self {
        match self {
            Query::Single(q) => match Query::new().not_side(side) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::Or, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.side = Some(Bool::IsNot(side));
                Query::Compound(q, LogicalCondition::Or, sub_query)
            }
        }
    }

    /// And search for any other side
    pub fn and_not_side(self, side: Side) -> Self {
        match self {
            Query::Single(q) => match Query::new().side(side) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::And, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.side = Some(Bool::Is(side));
                Query::Compound(q, LogicalCondition::And, sub_query)
            }
        }
    }

    /// Search for a particular player
    pub fn player(self, player: Option<&'a str>) -> Self {
        match self {
            Query::Single(mut sub_query) => {
                sub_query.player = Some(Bool::Is(player));
                Query::Single(sub_query)
            }
            Query::Compound(mut sub_query, op, q) => {
                sub_query.player = Some(Bool::Is(player));
                Query::Compound(sub_query, op, q)
            }
        }
    }

    /// Or search for a particular player
    pub fn or_player(self, player: Option<&'a str>) -> Self {
        match self {
            Query::Single(q) => match Query::new().player(player) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::Or, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.player = Some(Bool::Is(player));
                Query::Compound(q, LogicalCondition::Or, sub_query)
            }
        }
    }

    /// And search for a particular player
    pub fn and_player(self, player: Option<&'a str>) -> Self {
        match self {
            Query::Single(q) => match Query::new().player(player) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::And, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.player = Some(Bool::Is(player));
                Query::Compound(q, LogicalCondition::And, sub_query)
            }
        }
    }

    /// Search for any other player
    pub fn not_player(self, player: Option<&'a str>) -> Self {
        match self {
            Query::Single(mut sub_query) => {
                sub_query.player = Some(Bool::IsNot(player));
                Query::Single(sub_query)
            }
            Query::Compound(mut sub_query, op, q) => {
                sub_query.player = Some(Bool::IsNot(player));
                Query::Compound(sub_query, op, q)
            }
        }
    }

    /// Or search for any other player
    pub fn or_not_player(self, player: Option<&'a str>) -> Self {
        match self {
            Query::Single(q) => match Query::new().not_player(player) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::Or, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.player = Some(Bool::IsNot(player));
                Query::Compound(q, LogicalCondition::Or, sub_query)
            }
        }
    }

    /// And search for any other player
    pub fn and_not_player(self, player: Option<&'a str>) -> Self {
        match self {
            Query::Single(q) => match Query::new().player(player) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::And, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.player = Some(Bool::Is(player));
                Query::Compound(q, LogicalCondition::And, sub_query)
            }
        }
    }

    /// Search for a particular part
    pub fn part(self, part: Option<&'a str>) -> Self {
        match self {
            Query::Single(mut sub_query) => {
                sub_query.part = Some(Bool::Is(part));
                Query::Single(sub_query)
            }
            Query::Compound(mut sub_query, op, q) => {
                sub_query.part = Some(Bool::Is(part));
                Query::Compound(sub_query, op, q)
            }
        }
    }

    /// Or search for a particular part
    pub fn or_part(self, part: Option<&'a str>) -> Self {
        match self {
            Query::Single(q) => match Query::new().part(part) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::Or, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.part = Some(Bool::Is(part));
                Query::Compound(q, LogicalCondition::Or, sub_query)
            }
        }
    }

    /// And search for a particular part
    pub fn and_part(self, part: Option<&'a str>) -> Self {
        match self {
            Query::Single(q) => match Query::new().part(part) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::And, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.part = Some(Bool::Is(part));
                Query::Compound(q, LogicalCondition::And, sub_query)
            }
        }
    }

    /// Search for any other part
    pub fn not_part(self, part: Option<&'a str>) -> Self {
        match self {
            Query::Single(mut sub_query) => {
                sub_query.part = Some(Bool::IsNot(part));
                Query::Single(sub_query)
            }
            Query::Compound(mut sub_query, op, q) => {
                sub_query.part = Some(Bool::IsNot(part));
                Query::Compound(sub_query, op, q)
            }
        }
    }

    /// Or search for any other part
    pub fn or_not_part(self, part: Option<&'a str>) -> Self {
        match self {
            Query::Single(q) => match Query::new().not_part(part) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::Or, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.part = Some(Bool::IsNot(part));
                Query::Compound(q, LogicalCondition::Or, sub_query)
            }
        }
    }

    /// And search for any other part
    pub fn and_not_part(self, part: Option<&'a str>) -> Self {
        match self {
            Query::Single(q) => match Query::new().part(part) {
                Query::Single(sub_query) => Query::Compound(q, LogicalCondition::And, sub_query),
                _ => unreachable!(),
            },
            Query::Compound(q, _, mut sub_query) => {
                sub_query.part = Some(Bool::Is(part));
                Query::Compound(q, LogicalCondition::And, sub_query)
            }
        }
    }
}

impl<'a> Default for Query<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl Save {
    pub fn matches(&self, query: &Query) -> bool {
        match query {
            Query::Single(sub_query) => self.matches_sub_query(sub_query),
            Query::Compound(a, LogicalCondition::Or, b) => {
                self.matches_sub_query(a) || self.matches_sub_query(b)
            }
            Query::Compound(a, LogicalCondition::And, b) => {
                self.matches_sub_query(a) && self.matches_sub_query(b)
            }
        }
    }

    pub fn matches_sub_query(
        &self,
        SubQuery {
            turn,
            side,
            player,
            part,
        }: &SubQuery,
    ) -> bool {
        let turn_matches = match turn {
            Some(Bool::Is(TurnQuery::Single(turn))) => *turn == self.turn,
            Some(Bool::Is(TurnQuery::Inclusive(rng))) => rng.contains(&self.turn),
            Some(Bool::Is(TurnQuery::LowerBounded(rng))) => rng.contains(&self.turn),
            Some(Bool::Is(TurnQuery::UpperBounded(rng))) => rng.contains(&self.turn),
            Some(Bool::Is(TurnQuery::Unbounded(_))) => true,
            Some(Bool::IsNot(TurnQuery::Single(turn))) => *turn != self.turn,
            Some(Bool::IsNot(TurnQuery::Inclusive(rng))) => !rng.contains(&self.turn),
            Some(Bool::IsNot(TurnQuery::LowerBounded(rng))) => !rng.contains(&self.turn),
            Some(Bool::IsNot(TurnQuery::UpperBounded(rng))) => !rng.contains(&self.turn),
            Some(Bool::IsNot(TurnQuery::Unbounded(_))) => false,
            None => true,
        };

        let side_matches = side
            .as_ref()
            .map(|sub_query| match sub_query {
                Bool::Is(s) => *s == self.side,
                Bool::IsNot(s) => *s != self.side,
            })
            .unwrap_or(true);
        let player_matches = player
            .as_ref()
            .map(|sub_query| match sub_query {
                Bool::Is(p) => *p == self.player.as_deref(),
                Bool::IsNot(p) => *p != self.player.as_deref(),
            })
            .unwrap_or(true);
        let part_matches = part
            .as_ref()
            .map(|sub_query| match sub_query {
                Bool::Is(p) => *p == self.part.as_deref(),
                Bool::IsNot(p) => *p != self.part.as_deref(),
            })
            .unwrap_or(true);

        turn_matches && side_matches && player_matches && part_matches
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Bool<T> {
    Is(T),
    IsNot(T),
}

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

#[cfg(test)]
mod tests {
    use crate::interface::index::mock_index::MockIndex;
    use crate::interface::Index;

    use super::*;

    #[test]
    fn query_works() {
        use crate::{Save, Side};

        let saves = &[
            Save::new(Side::Allies, 1),
            Save::new(Side::Axis, 2),
            Save::new(Side::Allies, 3).player("A"),
            Save::new(Side::Axis, 4).player("B"),
            Save::new(Side::Allies, 5).player("A").part("1"),
        ];

        let mock_index = MockIndex::new(saves);

        for (query, expected_count) in &[
            (Query::new(), 5),
            (Query::new().side(Side::Allies), 3),
            (Query::new().side(Side::Allies).player(Some("A")), 2),
            (Query::new().side(Side::Allies).player(Some("B")), 0),
            (Query::new().side(Side::Axis).player(Some("A")), 0),
            (Query::new().side(Side::Axis).player(Some("B")), 1),
            (Query::new().side(Side::Axis).player(None), 1),
            (Query::new().part(None), 4),
            (Query::new().part(Some("1")), 1),
            (Query::new().turn(4), 1),
            (Query::new().turn_in_range(Some(2), Some(3)), 2),
            (Query::new().turn_in_range(None, Some(4)), 4),
            (Query::new().turn_in_range(Some(4), None), 2),
            (Query::new().turn_in_range(None, None), 5),
        ] {
            assert_eq!(
                mock_index.search(query).unwrap().len(),
                *expected_count,
                "query {query:?} had wrong count {expected_count}"
            );
        }
    }

    #[test]
    fn not_queries_work() {
        use crate::{Save, Side};

        let saves = &[
            Save::new(Side::Allies, 1),
            Save::new(Side::Axis, 2),
            Save::new(Side::Allies, 3).player("A"),
            Save::new(Side::Axis, 4).player("B"),
            Save::new(Side::Allies, 5).player("A").part("1"),
        ];

        let mock_index = MockIndex::new(saves);

        for (idx, (query, expected_count)) in [
            (Query::new().not_side(Side::Allies), 2),
            (Query::new().not_side(Side::Allies).player(Some("A")), 0),
            (Query::new().not_side(Side::Allies).player(Some("B")), 1),
            (Query::new().not_side(Side::Axis).player(Some("A")), 2),
            (Query::new().not_side(Side::Axis).player(Some("B")), 0),
            (Query::new().not_side(Side::Axis).player(None), 1),
            (Query::new().not_side(Side::Allies).not_player(Some("A")), 2),
            (Query::new().not_side(Side::Allies).not_player(Some("B")), 1),
            (Query::new().not_side(Side::Axis).not_player(Some("A")), 1),
            (Query::new().not_side(Side::Axis).not_player(Some("B")), 3),
            (Query::new().not_side(Side::Axis).not_player(None), 2),
            (Query::new().side(Side::Allies).not_player(Some("A")), 1),
            (Query::new().side(Side::Allies).not_player(Some("B")), 3),
            (Query::new().side(Side::Axis).not_player(Some("A")), 2),
            (Query::new().side(Side::Axis).not_player(Some("B")), 1),
            (Query::new().side(Side::Axis).not_player(None), 1),
            (Query::new().not_part(None), 1),
            (Query::new().not_part(Some("1")), 4),
            (Query::new().not_turn(4), 4),
            (Query::new().turn_not_in_range(Some(2), Some(3)), 3),
            (Query::new().turn_not_in_range(None, Some(4)), 1),
            (Query::new().turn_not_in_range(Some(4), None), 3),
            (Query::new().turn_not_in_range(None, None), 0),
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(
                mock_index.search(query).unwrap().len(),
                *expected_count,
                "test {idx}: query {query:?} had wrong count {expected_count}"
            );
        }
    }

    #[test]
    fn compound_queries_work() {
        use crate::{Save, Side};

        let saves = &[
            Save::new(Side::Allies, 1),
            Save::new(Side::Axis, 2),
            Save::new(Side::Allies, 3).player("A"),
            Save::new(Side::Axis, 4).player("B"),
            Save::new(Side::Allies, 5).player("A").part("1"),
        ];

        let mock_index = MockIndex::new(saves);

        for (idx, (query, expected_count)) in [
            (Query::new().side(Side::Allies).or_turn(2), 3),
            (
                Query::new()
                    .side(Side::Allies)
                    .player(Some("A"))
                    .or_side(Side::Axis)
                    .or_player(Some("B")),
                3,
            ),
            (
                Query::new()
                    .side(Side::Allies)
                    .player(None)
                    .turn(1)
                    .or_side(Side::Allies)
                    .or_not_player(None)
                    .or_turn_in_range(Some(4), Some(5)),
                2,
            ),
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(
                mock_index.search(query).unwrap().len(),
                *expected_count,
                "test {idx}: query {query:?} had wrong count {expected_count}"
            );
        }
    }
}
