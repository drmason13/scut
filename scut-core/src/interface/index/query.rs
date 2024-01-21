use compose::prelude::*;

use crate::{Save, Side};

use self::turn_number::TurnNumberRange;

mod builder;
mod impl_compose;
mod part;
mod player;
mod side;
mod turn;
mod turn_number;

/// Used to search for saves by specifying various conditions based on the turn, side, player and part of the save.
///
/// It is the input to methods of [`Index`] like [`search`]
///
/// [`Index`]: crate::interface::Index
/// [`search`]: crate::interface::Index::search
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Query<'a> {
    /// match the turn number either exactly or to within a range
    pub turn_number: Option<QueryParam<TurnNumberRange>>,
    /// match the side
    pub side: Option<QueryParam<Side>>,
    /// match the player - use `Some(None)` to match saves with no player (i.e. turn start saves)
    pub player: Option<QueryParam<Option<&'a str>>>,
    /// match the part - use `Some(None)` to match saves with no part
    pub part: Option<QueryParam<Option<&'a str>>>,
}

impl<'a> Query<'a> {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Implemented for [`Query`] and types wrapping [`Query`] so they can test whether they match a given [`Save`]
pub trait Matches<Value = Save> {
    fn matches(&self, value: &Value) -> bool;
}

impl<'a> Matches for Query<'a> {
    fn matches(&self, save: &Save) -> bool {
        let turn_number_matches = self
            .turn_number
            .as_ref()
            .map(|param| param.matches(&save.turn.number))
            .unwrap_or(true);

        let side_matches = self
            .side
            .as_ref()
            .map(|param| param.matches(&save.turn.side))
            .unwrap_or(true);

        let player_matches = self
            .player
            .as_ref()
            .map(|param| param.matches(&save.player.as_deref()))
            .unwrap_or(true);

        let part_matches = self
            .part
            .as_ref()
            .map(|param| param.matches(&save.part.as_deref()))
            .unwrap_or(true);

        turn_number_matches && side_matches && player_matches && part_matches
    }
}

impl<'a> Matches for Bool<Query<'a>> {
    fn matches(&self, save: &Save) -> bool {
        match self {
            Bool::Is(query) => query.matches(save),
            Bool::IsNot(query) => !query.matches(save),
        }
    }
}

impl<'a> Matches for Composable<Query<'a>> {
    fn matches(&self, save: &Save) -> bool {
        match self {
            Composable::Single(query) => query.matches(save),
            Composable::Compound(a, Op::Or, b) => a.matches(save) || b.matches(save),
            Composable::Compound(a, Op::And, b) => a.matches(save) && b.matches(save),
            Composable::Nested(a, Op::Or, b) => a.matches(save) || b.matches(save),
            Composable::Nested(a, Op::And, b) => a.matches(save) && b.matches(save),
        }
    }
}

impl<'a> Matches for Bool<Composable<Query<'a>>> {
    fn matches(&self, save: &Save) -> bool {
        match self {
            Bool::Is(query) => query.matches(save),
            Bool::IsNot(query) => !query.matches(save),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryParam<T> {
    Single(Bool<T>),
    Multi(Vec<Bool<T>>),
}

impl<T> QueryParam<T> {
    fn is(value: T) -> Self {
        QueryParam::Single(Bool::Is(value))
    }

    fn not(value: T) -> Self {
        QueryParam::Single(Bool::IsNot(value))
    }

    fn or(self, other: T) -> Self {
        match self {
            QueryParam::Single(x) => QueryParam::Multi(vec![x, Bool::Is(other)]),
            QueryParam::Multi(mut xs) => {
                xs.push(Bool::Is(other));
                QueryParam::Multi(xs)
            }
        }
    }

    fn or_not(self, other: T) -> Self {
        match self {
            QueryParam::Single(x) => QueryParam::Multi(vec![x, Bool::IsNot(other)]),
            QueryParam::Multi(mut xs) => {
                xs.push(Bool::IsNot(other));
                QueryParam::Multi(xs)
            }
        }
    }
}

impl<T> Matches<T> for QueryParam<T>
where
    T: PartialEq,
{
    fn matches(&self, value: &T) -> bool {
        match self {
            QueryParam::Single(x) => x.matches(value),
            QueryParam::Multi(xs) => xs.iter().any(|x| x.matches(value)),
        }
    }
}

impl<T> Matches<T> for Bool<T>
where
    T: PartialEq,
{
    fn matches(&self, value: &T) -> bool {
        match self {
            Bool::Is(x) => x == value,
            Bool::IsNot(x) => x != value,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interface::index::mock_index::MockIndex;
    use crate::interface::Index;
    use crate::{Save, Side, Turn};
    use compose::prelude::*;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn query_works() {
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
        let saves = &[
            Save::new(Side::Allies, 1),
            Save::new(Side::Axis, 2),
            Save::new(Side::Allies, 3).player("A"),
            Save::new(Side::Axis, 4).player("B"),
            Save::new(Side::Allies, 5).player("A").part("1"),
        ];

        let mock_index = MockIndex::new(saves);

        for (idx, (query, expected_count)) in [
            (
                Query::new()
                    .side(Side::Allies)
                    .or(Query::new().turn_number(2)),
                4,
            ),
            (
                Query::new()
                    .side(Side::Allies)
                    .player(Some("A"))
                    .or(Query::new().side(Side::Axis).player(Some("B"))),
                3,
            ),
            (
                Query::new()
                    .side(Side::Allies)
                    .player(None)
                    .turn_number(1)
                    .or(Query::new()
                        .side(Side::Allies)
                        .not_player(None)
                        .turn_number_in_range(Some(4), Some(5))),
                2,
            ),
            (
                Query::new().not_player(None).not_turn_number(3).and(
                    Query::new()
                        .not_player(Some("A"))
                        .turn_number_in_range(Some(4), Some(5)),
                ),
                1,
            ),
            (
                Query::new()
                    .not_player(None)
                    .or(Query::new().player(Some("A")).turn_number(3)),
                3,
            ),
            (
                Query::new()
                    .player(Some("A"))
                    .or_player(Some("B"))
                    .turn_number_in_range(None, Some(1))
                    .or_turn_number_not_in_range(Some(4), Some(5))
                    .or(Query::new().turn_number(99)),
                1,
            ),
            (
                Query::new()
                    .player(None)
                    .turn_number_in_range(None, Some(1))
                    .or_player(Some("A"))
                    .or(Query::new()
                        .player(None)
                        .or_player(Some("A"))
                        .turn_number_not_in_range(Some(4), Some(5))),
                3,
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
            .or(Query::new().side(Side::Axis).player(None).turn_number(1));

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
            .or(Query::new().side(Side::Axis).player(None).turn_number(2));

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

    #[test]
    fn test_query_by_turn() -> anyhow::Result<()> {
        let saves = &[
            Save::new(Side::Allies, 1),
            Save::new(Side::Axis, 2),
            Save::new(Side::Allies, 3).player("A"),
            Save::new(Side::Axis, 4).player("B"),
            Save::new(Side::Allies, 5).player("A").part("1"),
        ];

        let mock_index = MockIndex::new(saves);

        for (idx, (query, expected_count)) in [
            (Bool::Is(Query::new().turn(Turn::new(Side::Allies, 3))), 1),
            (Bool::Is(Query::new().turn(Turn::new(Side::Allies, 4))), 0),
            (Bool::Is(Query::new().turn(Turn::new(Side::Axis, 4))), 1),
            (Query::new().not_turn(Turn::new(Side::Axis, 2)), 4),
            (Query::new().not_turn(Turn::new(Side::Allies, 2)), 5),
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(
                mock_index.count(query).unwrap(),
                *expected_count,
                "test {idx}: query {query:?} had wrong count {expected_count}"
            );
        }

        Ok(())
    }
}
