pub(crate) mod query;

#[macro_export]
macro_rules! is {
    ($expression:expr) => {
        $crate::Bool::Is($expression)
    };
}

#[macro_export]
macro_rules! not {
    ($expression:expr) => {
        $crate::Bool::IsNot($expression)
    };
}

#[macro_export]
macro_rules! bx {
    ($expression:expr) => {
        Box::new($expression)
    };
}

pub mod prelude {
    pub use super::{And, Bool, Composable, Not, Op, Or};
}

pub trait Compose: And + Or + Not + Sized {}

pub trait And<Rhs = Self>
where
    Rhs: Compose,
{
    type Output: Compose;

    fn and(self, rhs: Rhs) -> Self::Output;
}

pub trait Or<Rhs = Self>
where
    Rhs: Compose,
{
    type Output: Compose;

    fn or(self, rhs: Rhs) -> Self::Output;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Bool<T> {
    Is(T),
    IsNot(T),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Composable<T> {
    Single(Bool<T>),
    Compound(Bool<T>, Op, Bool<T>),
    Nested(Box<Bool<Composable<T>>>, Op, Box<Bool<Composable<T>>>),
}

impl<T> Composable<T> {
    pub fn new(thing: T) -> Self {
        Composable::Single(is!(thing))
    }
}

impl<T> And for Composable<T> {
    type Output = Composable<T>;

    fn and(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Composable::Single(lhs), Composable::Single(rhs)) => {
                Composable::Compound(lhs, Op::And, rhs)
            }
            (lhs, rhs) => Composable::Nested(bx!(is!(lhs)), Op::And, bx!(is!(rhs))),
        }
    }
}

impl<T> Or for Composable<T> {
    type Output = Composable<T>;

    fn or(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Composable::Single(lhs), Composable::Single(rhs)) => {
                Composable::Compound(lhs, Op::Or, rhs)
            }
            (lhs, rhs) => Composable::Nested(bx!(is!(lhs)), Op::Or, bx!(is!(rhs))),
        }
    }
}

pub trait Not {
    type Output: Not;

    fn not(self) -> Self::Output;
}

impl<T> Not for Bool<T> {
    type Output = Bool<T>;

    fn not(self) -> Self::Output {
        match self {
            Bool::Is(t) => Bool::IsNot(t),
            Bool::IsNot(t) => Bool::Is(t),
        }
    }
}

impl<T> Not for Composable<T> {
    type Output = Bool<Composable<T>>;

    fn not(self) -> Self::Output {
        Bool::IsNot(self)
    }
}

impl<T> And for Bool<Composable<T>> {
    type Output = Composable<T>;

    fn and(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Bool::Is(Composable::Single(a)), Bool::Is(Composable::Single(b))) => {
                Composable::Compound(a, Op::And, b)
            }
            (Bool::IsNot(Composable::Single(a)), Bool::IsNot(Composable::Single(b))) => {
                Composable::Compound(a.not(), Op::And, b.not())
            }
            (Bool::Is(Composable::Single(a)), Bool::IsNot(Composable::Single(b))) => {
                Composable::Compound(a, Op::And, b.not())
            }
            (Bool::IsNot(Composable::Single(a)), Bool::Is(Composable::Single(b))) => {
                Composable::Compound(a.not(), Op::And, b)
            }

            (Bool::Is(lhs), Bool::Is(rhs)) => {
                Composable::Nested(bx!(is!(lhs)), Op::And, bx!(is!(rhs)))
            }
            (Bool::IsNot(lhs), Bool::IsNot(rhs)) => {
                Composable::Nested(bx!(not!(lhs)), Op::And, bx!(not!(rhs)))
            }

            (Bool::Is(lhs), Bool::IsNot(rhs)) => {
                Composable::Nested(bx!(is!(lhs)), Op::And, bx!(not!(rhs)))
            }

            (Bool::IsNot(lhs), Bool::Is(rhs)) => {
                Composable::Nested(bx!(not!(lhs)), Op::And, bx!(is!(rhs)))
            }
        }
    }
}

impl<T> Or for Bool<Composable<T>> {
    type Output = Composable<T>;

    fn or(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Bool::Is(Composable::Single(a)), Bool::Is(Composable::Single(b))) => {
                Composable::Compound(a, Op::Or, b)
            }
            (Bool::IsNot(Composable::Single(a)), Bool::IsNot(Composable::Single(b))) => {
                Composable::Compound(a.not(), Op::Or, b.not())
            }
            (Bool::Is(Composable::Single(a)), Bool::IsNot(Composable::Single(b))) => {
                Composable::Compound(a, Op::Or, b.not())
            }
            (Bool::IsNot(Composable::Single(a)), Bool::Is(Composable::Single(b))) => {
                Composable::Compound(a.not(), Op::Or, b)
            }

            (Bool::Is(lhs), Bool::Is(rhs)) => {
                Composable::Nested(bx!(is!(lhs)), Op::Or, bx!(is!(rhs)))
            }
            (Bool::IsNot(lhs), Bool::IsNot(rhs)) => {
                Composable::Nested(bx!(not!(lhs)), Op::Or, bx!(not!(rhs)))
            }

            (Bool::Is(lhs), Bool::IsNot(rhs)) => {
                Composable::Nested(bx!(is!(lhs)), Op::Or, bx!(not!(rhs)))
            }

            (Bool::IsNot(lhs), Bool::Is(rhs)) => {
                Composable::Nested(bx!(not!(lhs)), Op::Or, bx!(is!(rhs)))
            }
        }
    }
}

impl<T> And<Composable<T>> for Bool<Composable<T>> {
    type Output = Composable<T>;

    fn and(self, rhs: Composable<T>) -> Self::Output {
        match (self, rhs) {
            (Bool::Is(a), b) => a.and(b),
            (Bool::IsNot(a), b) => not!(a).and(is!(b)),
        }
    }
}

impl<T> And<Bool<Composable<T>>> for Composable<T> {
    type Output = Composable<T>;

    fn and(self, rhs: Bool<Composable<T>>) -> Self::Output {
        match (self, rhs) {
            (a, Bool::Is(b)) => a.and(b),
            (a, Bool::IsNot(b)) => is!(a).and(not!(b)),
        }
    }
}

impl<T> Or<Composable<T>> for Bool<Composable<T>> {
    type Output = Composable<T>;

    fn or(self, rhs: Composable<T>) -> Self::Output {
        match (self, rhs) {
            (Bool::Is(a), b) => a.or(b),
            (Bool::IsNot(a), b) => not!(a).or(is!(b)),
        }
    }
}

impl<T> Or<Bool<Composable<T>>> for Composable<T> {
    type Output = Composable<T>;

    fn or(self, rhs: Bool<Composable<T>>) -> Self::Output {
        match (self, rhs) {
            (a, Bool::Is(b)) => a.or(b),
            (a, Bool::IsNot(b)) => is!(a).or(not!(b)),
        }
    }
}

impl<T> Compose for Bool<Composable<T>> {}

impl<T> Compose for Composable<T> {}

#[cfg(test)]
mod tests {
    use crate::query::Query;

    use super::prelude::*;

    #[test]
    fn it_works() {
        let f = || Composable::new(Query);

        let a_or_b = f().or(f());
        let a_or_b_and_b = {
            let a_or_b = f().or(f());
            a_or_b.and(f())
        };

        assert!(
            matches!(a_or_b, Composable::Compound(_, Op::Or, _)),
            "expected Composable::Compound(_, Op::Or, _), got {a_or_b:?}"
        );

        assert!(
            matches!(a_or_b_and_b, Composable::Nested(_, Op::And, _)),
            "expected Composable::Nested(_, Op::And, _), got {a_or_b_and_b:?}"
        );
    }

    #[test]
    fn not_works() {
        let f = || Composable::new(Query);

        let a_or_b = f().or(f());

        let not_a_or_b = a_or_b.not();

        assert!(
            matches!(not_a_or_b, Bool::IsNot(Composable::Compound(_, Op::Or, _))),
            "expected Bool::Is(Composable::Compound(_, Op::Or, _)), got {not_a_or_b:?}"
        );

        let not_not_a_or_b = not_a_or_b.not();

        assert!(
            matches!(not_not_a_or_b, Bool::Is(Composable::Compound(_, Op::Or, _))),
            "expected Bool::Is(Composable::Compound(_, Op::Or, _)), got {not_not_a_or_b:?}"
        );

        let a_and_b = f().and(f());
        let a_or_not_a_and_b = f().or(a_and_b.not());
        let diagnostic = a_or_not_a_and_b.clone();

        assert!(
            matches!(a_or_not_a_and_b, Composable::Nested(_, Op::Or, bx) if matches!(*bx, Bool::IsNot(_))),
            "expected Composable::Nested(_, Op::Or, Bool::IsNot(_)), got {diagnostic:?}"
        );

        let a_and_b = f().and(f());
        let not_a_or_not_a_and_b = f().or(a_and_b.not()).not();
        let diagnostic = not_a_or_not_a_and_b.clone();

        assert!(
            matches!(
                not_a_or_not_a_and_b,
                Bool::IsNot(Composable::Nested(_, Op::Or, bx)) if matches!(*bx, Bool::IsNot(_)),
            ),
            "expected Bool::IsNot(Composable::Nested(_, Op::Or, Bool::IsNot(_))), got {diagnostic:?}"
        );
    }
}
