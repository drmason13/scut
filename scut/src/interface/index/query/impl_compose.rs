use super::Query;
use compose::{bx, is, not, prelude::*, Compose};

impl<'a> Compose for Query<'a> {}

impl<'a> And<Query<'a>> for Query<'a> {
    type Output = Composable<Query<'a>>;

    fn and(self, rhs: Self) -> Self::Output {
        Composable::Compound(is!(self), Op::And, is!(rhs))
    }
}

impl<'a> Or<Query<'a>> for Query<'a> {
    type Output = Composable<Query<'a>>;

    fn or(self, rhs: Self) -> Self::Output {
        Composable::Compound(is!(self), Op::Or, is!(rhs))
    }
}

impl<'a> Not for Query<'a> {
    type Output = Bool<Query<'a>>;

    fn not(self) -> Self::Output {
        not!(self)
    }
}

impl<'a> And<Composable<Query<'a>>> for Query<'a> {
    type Output = Composable<Query<'a>>;

    fn and(self, rhs: Composable<Query<'a>>) -> Self::Output {
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

impl<'a> Or<Composable<Query<'a>>> for Query<'a> {
    type Output = Composable<Query<'a>>;

    fn or(self, rhs: Composable<Query<'a>>) -> Self::Output {
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

impl<'a> And<Query<'a>> for Composable<Query<'a>> {
    type Output = Composable<Query<'a>>;

    fn and(self, rhs: Query<'a>) -> Self::Output {
        match self {
            Composable::Single(lhs) => Composable::Compound(lhs, Op::And, is!(rhs)),
            lhs => Composable::Nested(
                bx!(is!(lhs)),
                Op::And,
                bx!(is!(Composable::Single(is!(rhs)))),
            ),
        }
    }
}

impl<'a> Or<Query<'a>> for Composable<Query<'a>> {
    type Output = Composable<Query<'a>>;

    fn or(self, rhs: Query<'a>) -> Self::Output {
        match self {
            Composable::Single(lhs) => Composable::Compound(lhs, Op::Or, is!(rhs)),
            lhs => Composable::Nested(
                bx!(is!(lhs)),
                Op::Or,
                bx!(is!(Composable::Single(is!(rhs)))),
            ),
        }
    }
}
