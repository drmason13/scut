use std::cmp::Ordering;

use crate::Turn;

use super::{builder::QueryBuildParameter, Query, SubQuery, TurnNumberQuery};

pub struct TurnQuery {
    order: Ordering,
    turn: Turn,
}

impl<'a> QueryBuildParameter<'a> for TurnQuery {
    fn new_sub_query(self, boolean: bool) -> SubQuery<'a> {
        let sub_query = self.turn.side.new_sub_query(boolean);
        match self.order {
            Ordering::Equal => TurnNumberQuery::Single(self.turn.number),
            Ordering::Greater => TurnNumberQuery::LowerBounded(self.turn.number..),
            Ordering::Less => TurnNumberQuery::UpperBounded(..=self.turn.number),
        }
        .merge_into(sub_query, boolean)
    }

    fn merge_into(self, sub_query: SubQuery<'a>, boolean: bool) -> SubQuery<'a> {
        match self.order {
            Ordering::Equal => TurnNumberQuery::Single(self.turn.number),
            Ordering::Greater => TurnNumberQuery::LowerBounded(self.turn.number..),
            Ordering::Less => TurnNumberQuery::UpperBounded(..=self.turn.number),
        }
        .merge_into(self.turn.side.merge_into(sub_query, boolean), boolean)
    }
}

impl<'a> Query<'a> {
    /// Search for a particular turn
    pub fn turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Equal,
            turn,
        };
        turn.build(self)
    }

    /// Or search for a particular turn
    pub fn or_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Equal,
            turn,
        };
        turn.build_or(self)
    }

    /// And search for a particular turn
    pub fn and_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Equal,
            turn,
        };
        turn.build_and(self)
    }

    /// Search for any other turn
    pub fn not_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Equal,
            turn,
        };
        turn.build_not(self)
    }

    /// Or search for any other turn
    pub fn or_not_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Equal,
            turn,
        };
        turn.build_or_not(self)
    }

    /// And search for any other turn
    pub fn and_not_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Equal,
            turn,
        };
        turn.build_and_not(self)
    }

    /// Search for a particular turn, or a later turn from the same side
    pub fn min_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Greater,
            turn,
        };
        turn.build(self)
    }

    /// Or search for a particular turn, or a later turn from the same side
    pub fn or_min_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Greater,
            turn,
        };
        turn.build_or(self)
    }

    /// And search for a particular turn, or a later turn from the same side
    pub fn and_min_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Greater,
            turn,
        };
        turn.build_and(self)
    }

    /// Search for any other turn / later turn from the same side
    pub fn not_min_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Greater,
            turn,
        };
        turn.build_not(self)
    }

    /// Or search for any other turn / later turn from the same side
    pub fn or_not_min_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Greater,
            turn,
        };
        turn.build_or_not(self)
    }

    /// And search for any other turn / later turn from the same side
    pub fn and_not_min_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Greater,
            turn,
        };
        turn.build_and_not(self)
    }

    /// Search for a particular turn, or a later turn from the same side
    pub fn max_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Less,
            turn,
        };
        turn.build(self)
    }

    /// Or search for a particular turn, or a later turn from the same side
    pub fn or_max_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Less,
            turn,
        };
        turn.build_or(self)
    }

    /// And search for a particular turn, or a later turn from the same side
    pub fn and_max_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Less,
            turn,
        };
        turn.build_and(self)
    }

    /// Search for any other turn / later turn from the same side
    pub fn not_max_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Less,
            turn,
        };
        turn.build_not(self)
    }

    /// Or search for any other turn / later turn from the same side
    pub fn or_not_max_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Less,
            turn,
        };
        turn.build_or_not(self)
    }

    /// And search for any other turn / later turn from the same side
    pub fn and_not_max_turn(self, turn: Turn) -> Self {
        let turn = TurnQuery {
            order: Ordering::Less,
            turn,
        };
        turn.build_and_not(self)
    }
}
