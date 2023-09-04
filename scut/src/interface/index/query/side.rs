use crate::Side;

use super::{Bool, LogicalCondition, Query};

impl<'a> Query<'a> {
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
}
