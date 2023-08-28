use crate::{bx, is, not, And, Bool, Composable, Compose, Not, Op, Or};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Query;

impl Compose for Query {}

impl And<Query> for Query {
    type Output = Composable<Query>;

    fn and(self, rhs: Self) -> Self::Output {
        Composable::Compound(is!(self), Op::And, is!(rhs))
    }
}

impl Or<Query> for Query {
    type Output = Composable<Query>;

    fn or(self, rhs: Self) -> Self::Output {
        Composable::Compound(is!(self), Op::Or, is!(rhs))
    }
}

impl Not for Query {
    type Output = Bool<Query>;

    fn not(self) -> Self::Output {
        not!(self)
    }
}

impl And<Composable<Query>> for Query {
    type Output = Composable<Query>;

    fn and(self, rhs: Composable<Query>) -> Self::Output {
        match rhs {
            Composable::Single(rhs) => Composable::Compound(is!(self), Op::And, rhs),
            rhs => Composable::Nested(
                bx!(is!(Composable::Single(is!(self)))),
                Op::And,
                bx!(is!(rhs)),
            ),
        }
    }
}

impl Or<Composable<Query>> for Query {
    type Output = Composable<Query>;

    fn or(self, rhs: Composable<Query>) -> Self::Output {
        match rhs {
            Composable::Single(rhs) => Composable::Compound(is!(self), Op::Or, rhs),
            rhs => Composable::Nested(
                bx!(is!(Composable::Single(is!(self)))),
                Op::Or,
                bx!(is!(rhs)),
            ),
        }
    }
}
