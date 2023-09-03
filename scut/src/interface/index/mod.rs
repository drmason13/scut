//! The [`Index`] interface allows searching for [`Save`]s.
//!
//! [`storage`](crate::interface::storage) interfaces provide a compatible implementation of [`Index`]
//! to allow searching within their store of saves via the [`index`](crate::interface::LocalStorage::index) method.

use crate::{Save, Side};

pub use self::query::Query;

pub mod query;

#[cfg(test)]
pub mod mock_index;

/// The [`Index`] interface allows searching for [`Save`]s by turn, Side, player and/or part using a query.
/// As well as getting the earliest or latest turn for a [`Side`].
///
/// See [`IterIndex`] for an easy way to implement this trait. Any type of "iterable" can be suitable such as a Vec, or the keys or values of a HashMap.
pub trait Index<'a> {
    fn search(&'a self, query: &Query) -> anyhow::Result<Vec<Save>>;

    fn count(&'a self, query: &Query) -> anyhow::Result<usize>;

    /// Return the latest turn for a side, if it exists
    fn latest_save(&'a self, side: Side) -> anyhow::Result<Option<Save>>;

    /// Return the earliest turn for a side, if it exists
    fn earliest_save(&'a self, side: Side) -> anyhow::Result<Option<Save>>;
}

/// We can implement an index using any collection yielding [`&Save`](Save)s. This makes things very flexible!
///
/// Simply implement [`IterIndex`] to use this implementation.
impl<'a, T> Index<'a> for T
where
    T: IterIndex<'a>,
{
    fn search(&'a self, query: &Query) -> anyhow::Result<Vec<Save>> {
        Ok(self
            .iter()
            .filter(|save| save.matches(query))
            .cloned()
            .collect())
    }

    fn count(&'a self, query: &Query) -> anyhow::Result<usize> {
        Ok(self.iter().filter(|save| save.matches(query)).count())
    }

    fn latest_save(&'a self, side: Side) -> anyhow::Result<Option<Save>> {
        Ok(self
            .iter()
            .filter(|save| save.side == side)
            .cloned()
            .max_by_key(|save| save.turn))
    }

    fn earliest_save(&'a self, side: Side) -> anyhow::Result<Option<Save>> {
        Ok(self
            .iter()
            .filter(|save| save.side == side)
            .cloned()
            .min_by_key(|save| save.turn))
    }
}

/// A type that has an "iterable" of [`Save`]s, can easily implement this trait and implement [`Index`] for free, neat!
pub trait IterIndex<'a> {
    type Iter: Iterator<Item = &'a Save>;

    fn iter(&'a self) -> Self::Iter;
}
