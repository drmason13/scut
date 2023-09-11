use std::default::Default;

use crate::Side;

use super::{builder::QueryParam, Bool, Query, SubQuery};

#[derive(Debug, Clone, PartialEq)]
pub struct SideQueryParam {
    boolean: Bool,
    side: Side,
}

impl SideQueryParam {
    pub fn from_side(side: Side, boolean: Bool) -> Self {
        SideQueryParam { boolean, side }
    }
}

impl<'a> QueryParam<'a> for SideQueryParam {
    type Value = Side;

    fn matches(&self, value: Self::Value) -> bool {
        self.side == value
    }

    fn new_sub_query(self) -> SubQuery<'a> {
        SubQuery {
            side: Some(self),
            ..Default::default()
        }
    }

    fn merge_into(self, mut sub_query: SubQuery<'a>) -> SubQuery<'a> {
        sub_query.side = Some(self);
        sub_query
    }
}

impl<'a> Query<'a> {
    /// Search for a particular side
    pub fn side(self, side: Side) -> Self {
        let side = SideQueryParam {
            boolean: Bool::Is,
            side,
        };
        side.apply(self)
    }

    /// Or search for a particular side
    pub fn or_side(self, side: Side) -> Self {
        let side = SideQueryParam {
            boolean: Bool::Is,
            side,
        };
        side.apply_or(self)
    }

    /// And search for a particular side
    pub fn and_side(self, side: Side) -> Self {
        let side = SideQueryParam {
            boolean: Bool::Is,
            side,
        };
        side.apply_and(self)
    }

    /// Search for any other side
    pub fn not_side(self, side: Side) -> Self {
        let side = SideQueryParam {
            boolean: Bool::Not,
            side,
        };
        side.apply(self)
    }

    /// Or search for any other side
    pub fn or_not_side(self, side: Side) -> Self {
        let side = SideQueryParam {
            boolean: Bool::Not,
            side,
        };
        side.apply_or(self)
    }

    /// And search for any other side
    pub fn and_not_side(self, side: Side) -> Self {
        let side = SideQueryParam {
            boolean: Bool::Not,
            side,
        };
        side.apply_and(self)
    }
}
