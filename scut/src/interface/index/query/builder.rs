use super::{Bool, LogicalCondition, Query, SubQuery, AND, OR};

pub struct QueryBuilder<'a> {
    query: Query<'a>,
}

impl<'a> Query<'a> {
    pub fn builder() -> QueryBuilder<'a> {
        QueryBuilder::new()
    }
}

impl<'a> QueryBuilder<'a> {
    fn new() -> Self {
        QueryBuilder {
            query: Query::new(),
        }
    }

    fn build(self) -> Query<'a> {
        self.query
    }

    fn not<F>(mut self, f: F) -> Self
    where
        F: FnOnce(QueryBuilder<'a>) -> QueryBuilder<'a>,
    {
        let built = f(QueryBuilder::new());

        todo!()
    }

    fn or<F>(mut self, f: F) -> Self
    where
        F: FnOnce(QueryBuilder<'a>) -> QueryBuilder<'a>,
    {
        let built = f(QueryBuilder::new());

        todo!()
    }

    fn and<F>(mut self, f: F) -> Self
    where
        F: FnOnce(QueryBuilder<'a>) -> QueryBuilder<'a>,
    {
        let built = f(QueryBuilder::new());

        todo!()
    }

    fn or_not<F>(mut self, f: F) -> Self
    where
        F: FnOnce(QueryBuilder<'a>) -> QueryBuilder<'a>,
    {
        let built = f(QueryBuilder::new());

        todo!()
    }

    fn and_not<F>(mut self, f: F) -> Self
    where
        F: FnOnce(QueryBuilder<'a>) -> QueryBuilder<'a>,
    {
        let built = f(QueryBuilder::new());

        todo!()
    }
}

/// "Search parameters" will implement this trait by filling in the implementation
/// of [`new_sub_query`](QueryParam::new_sub_query) and [`merge_into`](QueryParam::merge_into)
/// to gain all the related `and`, `or` and `not` combinator methods for free
pub(super) trait QueryParam<'a>: Sized {
    type Value;

    fn matches(&self, value: Self::Value) -> bool;

    /// Create a new `SubQuery` that matches
    fn new_sub_query(self) -> SubQuery<'a>;

    fn merge_into(self, sub_query: SubQuery<'a>) -> SubQuery<'a>;

    /// Apply this search parameter to a query
    fn apply(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single(sub_query) => Query::Single(self.merge_into(sub_query)),
            Query::Compound(sub_query, op, q) => Query::Compound(self.merge_into(sub_query), op, q),
            Query::Nested(a, op, b) => todo!(),
        }
    }

    /// Or search for a particular part
    fn apply_or(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single(q) => Query::Compound(q, OR, self.new_sub_query()),
            Query::Compound(q, _, sub_query) => Query::Compound(q, OR, self.merge_into(sub_query)),
            Query::Nested(a, op, b) => todo!(),
        }
    }

    /// And search for a particular part
    fn apply_and(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single(q) => Query::Compound(q, AND, self.new_sub_query()),
            Query::Compound(q, _, sub_query) => Query::Compound(q, AND, self.merge_into(sub_query)),
            Query::Nested(a, op, b) => todo!(),
        }
    }

    /// Search for any other part
    fn apply_not(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single(sub_query) => Query::Single(self.merge_into(sub_query)),
            Query::Compound(sub_query, op, q) => Query::Compound(self.merge_into(sub_query), op, q),
            Query::Nested(a, op, b) => todo!(),
        }
    }

    /// Or search for any other part
    fn apply_or_not(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single(q) => Query::Compound(q, OR, self.new_sub_query()),
            Query::Compound(q, _, sub_query) => Query::Compound(q, OR, self.merge_into(sub_query)),
            Query::Nested(a, op, b) => todo!(),
        }
    }

    /// And search for any other part
    fn apply_and_not(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single(q) => Query::Compound(q, AND, self.new_sub_query()),
            Query::Compound(q, _, sub_query) => Query::Compound(q, AND, self.merge_into(sub_query)),
            Query::Nested(a, op, b) => todo!(),
        }
    }
}
