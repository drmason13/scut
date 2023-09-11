use crate::Save;

use self::{
    builder::QueryParam, part::PartQueryParam, player::PlayerQueryParam, side::SideQueryParam,
    turn_number::TurnNumberQueryParam,
};

mod builder;
mod part;
mod player;
mod side;
mod turn;
mod turn_number;

const OR: LogicalCondition = LogicalCondition::Or(Bool::Is);
const AND: LogicalCondition = LogicalCondition::And(Bool::Is);
const OR_NOT: LogicalCondition = LogicalCondition::Or(Bool::Not);
const AND_NOT: LogicalCondition = LogicalCondition::And(Bool::Not);

/// Used to search for saves by specifying various conditions based on the turn, side, player and part of the save.
///
/// It is the input to methods of [`Index`] like [`search`]
///
/// [`Index`]: crate::interface::Index
/// [`search`]: crate::interface::Index::search
#[derive(Debug, Clone, PartialEq)]
pub enum Query<'a> {
    Single {
        boolean: Bool,
        sub_query: SubQuery<'a>,
    },
    Compound {
        boolean: Bool,
        a: SubQuery<'a>,
        op: LogicalCondition,
        b: SubQuery<'a>,
    },
    Nested {
        boolean: Bool,
        a: Box<Query<'a>>,
        op: LogicalCondition,
        b: Box<Query<'a>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicalCondition {
    And(Bool),
    Or(Bool),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Bool {
    #[default]
    Is,
    Not,
}

impl Bool {
    /// Applies this to an existing `bool`, negating it if we are Bool::Not.
    ///
    /// ```
    /// # use crate::interface::index::query::Bool;
    /// assert!(Bool::Is.apply(true));
    /// assert!(Bool::Not.apply(false));
    ///
    /// assert!(!Bool::Is.apply(false));
    /// assert!(!Bool::Not.apply(true));
    /// ```
    pub fn apply(&self, boolean: bool) -> bool {
        match self {
            Bool::Is => boolean,
            Bool::Not => !boolean,
        }
    }

    /// Inverts this boolean.
    ///
    /// ```
    /// assert_eq!(Bool::Is.inverse(), Bool::IsNot);
    /// assert_eq!(Bool::Is.inverse().inverse(), Bool::Is);
    /// ```
    pub fn inverse(&self) -> Self {
        match self {
            Bool::Is => Bool::Not,
            Bool::Not => Bool::Is,
        }
    }
}

/// All the fields to filter [`Save`]s by, each field can be set to `None` to indicate a Wildcard search for that field
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SubQuery<'a> {
    pub boolean: Bool,
    /// match the turn number either exactly or to within a range
    pub turn_number: Option<TurnNumberQueryParam>,
    /// match the side
    pub side: Option<SideQueryParam>,
    /// match the player - use `Some(None)` to match saves with no player (i.e. turn start saves)
    pub player: Option<PlayerQueryParam<'a>>,
    /// match the part - use `Some(None)` to match saves with no part
    pub part: Option<PartQueryParam<'a>>,
}

impl<'a> Query<'a> {
    pub fn new() -> Self {
        Query::Single {
            boolean: Bool::Is,
            sub_query: SubQuery::default(),
        }
    }
}

impl Default for Query<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Save {
    #[rustfmt::skip]
    pub fn matches(&self, query: &Query) -> bool {
        match query {
            Query::Single   { boolean, sub_query }         => boolean.apply(self.matches_sub_query(sub_query)),
            Query::Compound { boolean, a, op: OR, b }      => boolean.apply(self.matches_sub_query(a) || self.matches_sub_query(b)),
            Query::Compound { boolean, a, op: OR_NOT, b }  => boolean.apply(self.matches_sub_query(a) || !self.matches_sub_query(b)),
            Query::Compound { boolean, a, op: AND, b }     => boolean.apply(self.matches_sub_query(a) && self.matches_sub_query(b)),
            Query::Compound { boolean, a, op: AND_NOT, b } => boolean.apply(self.matches_sub_query(a) && !self.matches_sub_query(b)),
            Query::Nested   { boolean, a, op: OR, b }      => boolean.apply(self.matches(a) || self.matches(b)),
            Query::Nested   { boolean, a, op: OR_NOT, b }  => boolean.apply(self.matches(a) || !self.matches(b)),
            Query::Nested   { boolean, a, op: AND, b }     => boolean.apply(self.matches(a) && self.matches(b)),
            Query::Nested   { boolean, a, op: AND_NOT, b } => boolean.apply(self.matches(a) && !self.matches(b)),
        }
    }

    pub fn matches_sub_query(
        &self,
        SubQuery {
            boolean,
            turn_number,
            side,
            player,
            part,
        }: &SubQuery,
    ) -> bool {
        let turn_number_matches = turn_number
            .as_ref()
            .map(|param| param.matches(self.turn.number))
            .unwrap_or(true);

        let side_matches = side
            .as_ref()
            .map(|param| param.matches(self.turn.side))
            .unwrap_or(true);

        let player_matches = player
            .as_ref()
            .map(|param| param.matches(self.player.as_deref()))
            .unwrap_or(true);

        let part_matches = part
            .as_ref()
            .map(|param| param.matches(self.part.as_deref()))
            .unwrap_or(true);

        let matches = turn_number_matches && side_matches && player_matches && part_matches;
        boolean.apply(matches)
    }
}

#[cfg(test)]
mod tests {
    use crate::interface::index::mock_index::MockIndex;
    use crate::interface::Index;
    use crate::{Save, Side, Turn};

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
            (Query::new().turn(Turn::new(Side::Allies, 3)), 1),
            (Query::new().turn(Turn::new(Side::Allies, 4)), 0),
            (Query::new().turn(Turn::new(Side::Axis, 4)), 1),
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

    fn nested_queries_work() -> anyhow::Result<()> {
        let turn = Turn::new(Side::Allies, 1);
        let query = Query::builder();
        todo!()
    }
}
