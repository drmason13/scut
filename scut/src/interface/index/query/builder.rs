use super::{Bool, LogicalCondition, Query, SubQuery, AND, OR, OR_NOT};

impl<'a> Query<'a> {
    pub fn not(self, other: Query<'a>) -> Self {
        match self {
            Query::Single {
                mut boolean,
                sub_query,
            } => Query::Single {
                boolean: boolean.inverse(),
                sub_query,
            },
            Query::Compound {
                mut boolean,
                mut a,
                op,
                mut b,
            } => Query::Compound {
                boolean: boolean.inverse(),
                a,
                op,
                b,
            },
            Query::Nested {
                mut boolean,
                mut a,
                op,
                mut b,
            } => Query::Nested {
                boolean: boolean.inverse(),
                a,
                op,
                b,
            },
        }
    }

    pub fn or(mut self, other: Query<'a>) -> Self {
        todo!()
    }

    pub fn and<F>(mut self, other: Query<'a>) -> Self {
        todo!()
    }

    pub fn or_not<F>(mut self, other: Query<'a>) -> Self {
        todo!()
    }

    pub fn and_not<F>(mut self, other: Query<'a>) -> Self {
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
            Query::Single { boolean, sub_query } => Query::Single {
                boolean,
                sub_query: self.merge_into(sub_query),
            },
            Query::Compound { boolean, a, op, b } => Query::Compound {
                boolean,
                a: self.merge_into(a),
                op,
                b,
            },
            Query::Nested { boolean, a, op, b } => todo!(),
        }
    }

    /// Or search for a particular part
    fn apply_or(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single { boolean, sub_query } => Query::Compound {
                boolean,
                a: sub_query,
                op: OR,
                b: self.new_sub_query(),
            },
            query @ Query::Compound { boolean, a, op, b } => Query::Nested {
                boolean,
                a: Box::new(query),
                op: OR,
                b: Box::new(Query::Single {
                    boolean: Bool::Is,
                    sub_query: self.new_sub_query(),
                }),
            },
            Query::Nested { boolean, a, op, b } => todo!(),
        }
    }

    /// And search for a particular part
    fn apply_and(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single { boolean, sub_query } => Query::Compound {
                boolean,
                a: sub_query,
                op: AND,
                b: self.new_sub_query(),
            },
            Query::Compound { boolean, a, op, b } => Query::Compound {
                boolean,
                a,
                op: AND,
                b,
            },
            Query::Nested { boolean, a, op, b } => todo!(),
        }
    }

    /// Search for any other part
    fn apply_not(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single { boolean, sub_query } => Query::Single {
                boolean,
                sub_query: self.merge_into(sub_query),
            },
            Query::Compound { boolean, a, op, b } => Query::Compound {
                boolean,
                a: self.merge_into(a),
                op,
                b,
            },
            Query::Nested { boolean, a, op, b } => todo!(),
        }
    }

    /// Or search for any other part
    fn apply_or_not(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single { boolean, sub_query } => Query::Compound {
                boolean,
                a: sub_query,
                op: OR_NOT,
                b: self.new_sub_query(),
            },
            Query::Compound { boolean, a, op, b } => Query::Compound {
                boolean,
                a,
                op: OR,
                b,
            },
            Query::Nested { boolean, a, op, b } => todo!(),
        }
    }

    /// And search for any other part
    fn apply_and_not(self, query: Query<'a>) -> Query<'a> {
        match query {
            Query::Single { boolean, sub_query } => Query::Compound {
                boolean,
                a,
                op: AND,
                b,
            },
            Query::Compound { boolean, a, op, b } => Query::Compound {
                boolean,
                a,
                op: AND,
                b,
            },
            Query::Nested { boolean, a, op, b } => todo!(),
        }
    }
}
