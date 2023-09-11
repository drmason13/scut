use std::default::Default;

use super::{builder::QueryParam, Bool, Query, SubQuery};

#[derive(Debug, Clone, PartialEq)]
pub struct PartQueryParam<'a> {
    boolean: Bool,
    part: Option<&'a str>,
}

impl<'a> PartQueryParam<'a> {
    pub fn from_side(part: Option<&'a str>, boolean: Bool) -> Self {
        PartQueryParam { boolean, part }
    }
}

impl<'a> QueryParam<'a> for PartQueryParam<'a> {
    type Value = Option<&'a str>;

    fn matches(&self, value: Self::Value) -> bool {
        self.part == value
    }

    fn new_sub_query(mut self) -> SubQuery<'a> {
        SubQuery {
            part: Some(self),
            ..Default::default()
        }
    }

    fn merge_into(self, mut sub_query: SubQuery<'a>) -> SubQuery<'a> {
        sub_query.part = Some(self);
        sub_query
    }
}

impl<'a> Query<'a> {
    /// Search for a particular part
    pub fn part(self, part: Option<&'a str>) -> Self {
        let part = PartQueryParam {
            part,
            boolean: Bool::Is,
        };
        part.apply(self)
    }

    /// Or search for a particular part
    pub fn or_part(self, part: Option<&'a str>) -> Self {
        let part = PartQueryParam {
            part,
            boolean: Bool::Is,
        };
        part.apply_or(self)
    }

    /// And search for a particular part
    pub fn and_part(self, part: Option<&'a str>) -> Self {
        let part = PartQueryParam {
            part,
            boolean: Bool::Is,
        };
        part.apply_and(self)
    }

    /// Search for any other part
    pub fn not_part(self, part: Option<&'a str>) -> Self {
        let part = PartQueryParam {
            part,
            boolean: Bool::Not,
        };
        part.apply_not(self)
    }

    /// Or search for any other part
    pub fn or_not_part(self, part: Option<&'a str>) -> Self {
        let part = PartQueryParam {
            part,
            boolean: Bool::Not,
        };
        part.apply_or_not(self)
    }

    /// And search for any other part
    pub fn and_not_part(self, part: Option<&'a str>) -> Self {
        let part = PartQueryParam {
            part,
            boolean: Bool::Not,
        };
        part.apply_and_not(self)
    }
}
