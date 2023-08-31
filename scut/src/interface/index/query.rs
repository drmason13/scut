use std::ops::{RangeFrom, RangeFull, RangeInclusive, RangeToInclusive};

use crate::{Save, Side};

/// All the fields to filter [`Save`]s by, each field can be set to `None` to indicate a Wildcard search for that field
#[derive(Debug, Clone, PartialEq)]
pub struct Query<'a> {
    /// match the [`TurnQuery`] (either a single turn or within a range)
    pub turn: Option<TurnQuery>,
    /// match the side
    pub side: Option<Side>,
    /// match the player - use `Some(None)` to match saves with no player (i.e. turn start saves)
    pub player: Option<Option<&'a str>>,
    /// match the part - use `Some(None)` to match saves with no part
    pub part: Option<Option<&'a str>>,
}

impl<'a> Query<'a> {
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
            (None, Some(b)) => Some(TurnQuery::UpperBounded(..=b)),
            (None, None) => None,
        };
        self
    }

    pub fn side(mut self, side: Side) -> Self {
        self.side = Some(side);
        self
    }

    pub fn player(mut self, player: Option<&'a str>) -> Self {
        self.player = Some(player);
        self
    }

    pub fn part(mut self, part: Option<&'a str>) -> Self {
        self.part = Some(part);
        self
    }
}

impl<'a> Default for Query<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl Save {
    pub fn matches(
        &self,
        Query {
            turn,
            side,
            player,
            part,
        }: &Query,
    ) -> bool {
        let turn_matches = match turn {
            Some(TurnQuery::Single(turn)) => *turn == self.turn,
            Some(TurnQuery::Inclusive(rng)) => rng.contains(&self.turn),
            Some(TurnQuery::LowerBounded(rng)) => rng.contains(&self.turn),
            Some(TurnQuery::UpperBounded(rng)) => rng.contains(&self.turn),
            Some(TurnQuery::Unbounded(_)) => true,
            None => true,
        };

        let side_matches = side.map(|s| s == self.side).unwrap_or(true);
        let player_matches = player
            .as_ref()
            .map(|p| *p == self.player.as_deref())
            .unwrap_or(true);
        let part_matches = part
            .as_ref()
            .map(|p| *p == self.part.as_deref())
            .unwrap_or(true);

        turn_matches && side_matches && player_matches && part_matches
    }
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
}
