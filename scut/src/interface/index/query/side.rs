use std::default::Default;

use crate::Side;

use super::{builder::QueryBuildParameter, Bool, Query, SubQuery};

impl<'a> QueryBuildParameter<'a> for Side {
    fn new_sub_query(self, boolean: bool) -> SubQuery<'a> {
        SubQuery {
            side: Some(if boolean {
                Bool::Is(self)
            } else {
                Bool::IsNot(self)
            }),
            ..Default::default()
        }
    }

    fn merge_into(self, mut sub_query: SubQuery<'a>, boolean: bool) -> SubQuery<'a> {
        sub_query.side = Some(if boolean {
            Bool::Is(self)
        } else {
            Bool::IsNot(self)
        });
        sub_query
    }
}

impl<'a> Query<'a> {
    /// Search for a particular side
    pub fn side(self, side: Side) -> Self {
        let side = side;
        side.build(self)
    }

    /// Or search for a particular side
    pub fn or_side(self, side: Side) -> Self {
        let side = side;
        side.build_or(self)
    }

    /// And search for a particular side
    pub fn and_side(self, side: Side) -> Self {
        let side = side;
        side.build_and(self)
    }

    /// Search for any other side
    pub fn not_side(self, side: Side) -> Self {
        let side = side;
        side.build_not(self)
    }

    /// Or search for any other side
    pub fn or_not_side(self, side: Side) -> Self {
        let side = side;
        side.build_or_not(self)
    }

    /// And search for any other side
    pub fn and_not_side(self, side: Side) -> Self {
        let side = side;
        side.build_and_not(self)
    }
}
