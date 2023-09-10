use crate::{Save, Side};

use self::turn_number::TurnNumberQuery;

mod builder;
mod part;
mod player;
mod side;
mod turn;
mod turn_number;

/// More complex nested compound queries could be supported, but the lifetimes become awkward and Boxes become required and it's not even needed.
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

#[derive(Debug, Clone, PartialEq)]
pub enum Bool<T> {
    Is(T),
    IsNot(T),
}

/// All the fields to filter [`Save`]s by, each field can be set to `None` to indicate a Wildcard search for that field
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SubQuery<'a> {
    /// match the [`TurnQuery`] (either a single turn_number or within a range)
    pub turn_number: Option<Bool<TurnNumberQuery>>,
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
            turn_number: None,
            side: None,
            player: None,
            part: None,
        })
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
            turn_number,
            side,
            player,
            part,
        }: &SubQuery,
    ) -> bool {
        let turn_number_matches = match turn_number {
            Some(Bool::Is(TurnNumberQuery::Single(turn_number))) => {
                *turn_number == self.turn.number
            }
            Some(Bool::Is(TurnNumberQuery::Inclusive(rng))) => rng.contains(&self.turn.number),
            Some(Bool::Is(TurnNumberQuery::LowerBounded(rng))) => rng.contains(&self.turn.number),
            Some(Bool::Is(TurnNumberQuery::UpperBounded(rng))) => rng.contains(&self.turn.number),
            Some(Bool::Is(TurnNumberQuery::Unbounded(_))) => true,
            Some(Bool::IsNot(TurnNumberQuery::Single(turn_number))) => {
                *turn_number != self.turn.number
            }
            Some(Bool::IsNot(TurnNumberQuery::Inclusive(rng))) => !rng.contains(&self.turn.number),
            Some(Bool::IsNot(TurnNumberQuery::LowerBounded(rng))) => {
                !rng.contains(&self.turn.number)
            }
            Some(Bool::IsNot(TurnNumberQuery::UpperBounded(rng))) => {
                !rng.contains(&self.turn.number)
            }
            Some(Bool::IsNot(TurnNumberQuery::Unbounded(_))) => false,
            None => true,
        };

        let side_matches = side
            .as_ref()
            .map(|sub_query| match sub_query {
                Bool::Is(s) => *s == self.turn.side,
                Bool::IsNot(s) => *s != self.turn.side,
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

        turn_number_matches && side_matches && player_matches && part_matches
    }
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
            (Query::new().turn_number(4), 1),
            (Query::new().turn_number_in_range(Some(2), Some(3)), 2),
            (Query::new().turn_number_in_range(None, Some(4)), 4),
            (Query::new().turn_number_in_range(Some(4), None), 2),
            (Query::new().turn_number_in_range(None, None), 5),
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
            (Query::new().not_turn_number(4), 4),
            (Query::new().turn_number_not_in_range(Some(2), Some(3)), 3),
            (Query::new().turn_number_not_in_range(None, Some(4)), 1),
            (Query::new().turn_number_not_in_range(Some(4), None), 3),
            (Query::new().turn_number_not_in_range(None, None), 0),
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
            (Query::new().side(Side::Allies).or_turn_number(2), 4),
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
                    .turn_number(1)
                    .or_side(Side::Allies)
                    .or_not_player(None)
                    .or_turn_number_in_range(Some(4), Some(5)),
                2,
            ),
            (
                Query::new()
                    .not_player(None)
                    .not_turn_number(3)
                    .and_not_player(Some("A"))
                    .and_turn_number_in_range(Some(4), Some(5)),
                1,
            ),
            (
                Query::new()
                    .not_player(None)
                    .turn_number(3)
                    .or_not_player(Some("A"))
                    .or_turn_number_not_in_range(Some(4), Some(5)),
                3,
            ),
            (
                Query::new()
                    .player(Some("A"))
                    .turn_number_in_range(None, Some(1))
                    .or_player(Some("B"))
                    .or_turn_number_not_in_range(Some(4), Some(5)),
                0,
            ),
            (
                Query::new()
                    .player(None)
                    .turn_number_in_range(None, Some(1))
                    .or_player(Some("A"))
                    .or_turn_number_not_in_range(Some(4), Some(5)),
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

    #[test]
    fn test_query_in_specific_scenario() -> anyhow::Result<()> {
        use crate::{Save, Side};

        let saves = &[
            Save::new(Side::Axis, 1),
            Save::new(Side::Axis, 1).player("DM"),
            Save::new(Side::Axis, 1).player("DG"),
            Save::new(Side::Axis, 2),
            Save::new(Side::Allies, 1),
            Save::new(Side::Allies, 1).player("GM"),
            Save::new(Side::Allies, 1).player("TG"),
        ];

        let mock_index = MockIndex::new(saves);
        let query = Query::new()
            .side(Side::Axis)
            .not_player(None)
            .turn_number_in_range(Some(1_u32.saturating_sub(1_u32)), None)
            .or_side(Side::Axis)
            .or_player(None)
            .or_turn_number(1);

        assert_eq!(
            mock_index.search(&query)?,
            vec![
                Save::new(Side::Axis, 1),
                Save::new(Side::Axis, 1).player("DM"),
                Save::new(Side::Axis, 1).player("DG"),
            ]
        );

        let query = Query::new()
            .side(Side::Axis)
            .not_player(None)
            .turn_number_in_range(Some(2_u32.saturating_sub(1_u32)), None)
            .or_side(Side::Axis)
            .or_player(None)
            .or_turn_number(2);

        assert_eq!(
            mock_index.search(&query)?,
            vec![
                Save::new(Side::Axis, 1).player("DM"),
                Save::new(Side::Axis, 1).player("DG"),
                Save::new(Side::Axis, 2),
            ]
        );
        Ok(())
    }
}
