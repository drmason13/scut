use super::{Bool, LogicalCondition, Query};

impl<'a> Query<'a> {
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
}
