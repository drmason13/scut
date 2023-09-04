use super::{Bool, LogicalCondition, Query};

impl<'a> Query<'a> {
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
