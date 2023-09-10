use super::{LogicalCondition, Query, SubQuery};

/// "Search parameters" will implement this trait by filling in the implementation
/// of [`new_sub_query`](QueryBuildParameter::new_sub_query) and [`merge_into`](QueryBuildParameter::merge_into)
/// to gain all the related `and`, `or` and `not` combinator methods for free
pub(super) trait QueryBuildParameter<'a>: Sized {
    /// Create a new `SubQuery` that matches
    fn new_sub_query(self, boolean: bool) -> SubQuery<'a>;

    fn merge_into(self, sub_query: SubQuery<'a>, boolean: bool) -> SubQuery<'a>;

    /// Apply this search parameter to a query
    fn build(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single(sub_query) => Query::Single(self.merge_into(sub_query, true)),
            Query::Compound(sub_query, op, q) => {
                Query::Compound(self.merge_into(sub_query, true), op, q)
            }
        }
    }

    /// Or search for a particular part
    fn build_or(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single(q) => Query::Compound(q, LogicalCondition::Or, self.new_sub_query(true)),
            Query::Compound(q, _, sub_query) => {
                Query::Compound(q, LogicalCondition::Or, self.merge_into(sub_query, true))
            }
        }
    }

    /// And search for a particular part
    fn build_and(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single(q) => Query::Compound(q, LogicalCondition::And, self.new_sub_query(true)),
            Query::Compound(q, _, sub_query) => {
                Query::Compound(q, LogicalCondition::And, self.merge_into(sub_query, true))
            }
        }
    }

    /// Search for any other part
    fn build_not(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single(sub_query) => Query::Single(self.merge_into(sub_query, false)),
            Query::Compound(sub_query, op, q) => {
                Query::Compound(self.merge_into(sub_query, false), op, q)
            }
        }
    }

    /// Or search for any other part
    fn build_or_not(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single(q) => Query::Compound(q, LogicalCondition::Or, self.new_sub_query(false)),
            Query::Compound(q, _, sub_query) => {
                Query::Compound(q, LogicalCondition::Or, self.merge_into(sub_query, false))
            }
        }
    }

    /// And search for any other part
    fn build_and_not(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single(q) => {
                Query::Compound(q, LogicalCondition::And, self.new_sub_query(false))
            }
            Query::Compound(q, _, sub_query) => {
                Query::Compound(q, LogicalCondition::And, self.merge_into(sub_query, false))
            }
        }
    }
}
