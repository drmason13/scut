use crate::Side;

use super::{Query, QueryParam};

impl<'a> Query<'a> {
    /// Search for a particular side
    pub fn side(mut self, side: Side) -> Self {
        let side = QueryParam::is(side);
        self.side = Some(side);
        self
    }

    /// Or search for a particular side
    pub fn or_side(mut self, side: Side) -> Self {
        self.side = self.side.map(|s| s.or(side));
        self
    }

    /// Search for any other side
    pub fn not_side(mut self, side: Side) -> Self {
        let side = QueryParam::not(side);
        self.side = Some(side);
        self
    }

    /// Or search for any other side
    pub fn or_not_side(mut self, side: Side) -> Self {
        self.side = self.side.map(|s| s.or_not(side));
        self
    }
}
