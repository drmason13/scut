use std::default::Default;

use super::{builder::QueryBuildParameter, Bool, Query, SubQuery};

struct Part<'a>(Option<&'a str>);

impl<'a> QueryBuildParameter<'a> for Part<'a> {
    fn new_sub_query(self, boolean: bool) -> SubQuery<'a> {
        SubQuery {
            part: Some(if boolean {
                Bool::Is(self.0)
            } else {
                Bool::IsNot(self.0)
            }),
            ..Default::default()
        }
    }

    fn merge_into(self, mut sub_query: SubQuery<'a>, boolean: bool) -> SubQuery<'a> {
        sub_query.part = Some(if boolean {
            Bool::Is(self.0)
        } else {
            Bool::IsNot(self.0)
        });
        sub_query
    }
}

impl<'a> Query<'a> {
    /// Search for a particular part
    pub fn part(self, part: Option<&'a str>) -> Self {
        let part = Part(part);
        part.build(self)
    }

    /// Or search for a particular part
    pub fn or_part(self, part: Option<&'a str>) -> Self {
        let part = Part(part);
        part.build_or(self)
    }

    /// And search for a particular part
    pub fn and_part(self, part: Option<&'a str>) -> Self {
        let part = Part(part);
        part.build_and(self)
    }

    /// Search for any other part
    pub fn not_part(self, part: Option<&'a str>) -> Self {
        let part = Part(part);
        part.build_not(self)
    }

    /// Or search for any other part
    pub fn or_not_part(self, part: Option<&'a str>) -> Self {
        let part = Part(part);
        part.build_or_not(self)
    }

    /// And search for any other part
    pub fn and_not_part(self, part: Option<&'a str>) -> Self {
        let part = Part(part);
        part.build_and_not(self)
    }
}
