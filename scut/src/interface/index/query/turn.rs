use std::cmp::Ordering;

use crate::Turn;

use super::{builder::QueryParam, Bool, Query, SubQuery};

#[derive(Debug, Clone, PartialEq)]
pub struct TurnQueryParam {
    order: Ordering,
    turn: Turn,
    boolean: Bool,
}

impl<'a> QueryParam<'a> for TurnQueryParam {
    type Value = Turn;

    fn matches(&self, value: Self::Value) -> bool {
        let matches = match self.order {
            Ordering::Equal => self.turn == value,
            Ordering::Greater => self.turn >= value,
            Ordering::Less => self.turn <= value,
        };
        self.boolean.apply(matches)
    }

    fn new_sub_query(self) -> SubQuery<'a> {
        SubQuery {
            side: Some(self.turn.side.into()),
            turn_number: Some(self.turn.number.into()),
            ..Default::default()
        }
    }

    fn merge_into(self, mut sub_query: SubQuery<'a>) -> SubQuery<'a> {
        sub_query.side = Some(self.turn.side.into());
        sub_query.turn_number = Some(self.turn.number.into());
        sub_query
    }
}

impl<'a> Query<'a> {
    /// Search for a particular turn
    pub fn turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Is,
            order: Ordering::Equal,
            turn,
        };
        turn.apply(self)
    }

    /// Or search for a particular turn
    pub fn or_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Is,
            order: Ordering::Equal,
            turn,
        };
        turn.apply_or(self)
    }

    /// And search for a particular turn
    pub fn and_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Is,
            order: Ordering::Equal,
            turn,
        };
        turn.apply_and(self)
    }

    /// Search for any other turn
    pub fn not_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Not,
            order: Ordering::Equal,
            turn,
        };
        turn.apply(self)
    }

    /// Or search for any other turn
    pub fn or_not_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Not,
            order: Ordering::Equal,
            turn,
        };
        turn.apply_or(self)
    }

    /// And search for any other turn
    pub fn and_not_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Not,
            order: Ordering::Equal,
            turn,
        };
        turn.apply_and(self)
    }

    /// Search for a particular turn, or a later turn from the same side
    pub fn min_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Is,
            order: Ordering::Greater,
            turn,
        };
        turn.apply(self)
    }

    /// Or search for a particular turn, or a later turn from the same side
    pub fn or_min_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Is,
            order: Ordering::Greater,
            turn,
        };
        turn.apply_or(self)
    }

    /// And search for a particular turn, or a later turn from the same side
    pub fn and_min_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Is,
            order: Ordering::Greater,
            turn,
        };
        turn.apply_and(self)
    }

    /// Search for any other turn / later turn from the same side
    pub fn not_min_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Not,
            order: Ordering::Greater,
            turn,
        };
        turn.apply(self)
    }

    /// Or search for any other turn / later turn from the same side
    pub fn or_not_min_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Not,
            order: Ordering::Greater,
            turn,
        };
        turn.apply_or(self)
    }

    /// And search for any other turn / later turn from the same side
    pub fn and_not_min_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Not,
            order: Ordering::Greater,
            turn,
        };
        turn.apply_and(self)
    }

    /// Search for a particular turn, or a later turn from the same side
    pub fn max_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Is,
            order: Ordering::Less,
            turn,
        };
        turn.apply(self)
    }

    /// Or search for a particular turn, or a later turn from the same side
    pub fn or_max_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Is,
            order: Ordering::Less,
            turn,
        };
        turn.apply_or(self)
    }

    /// And search for a particular turn, or a later turn from the same side
    pub fn and_max_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Is,
            order: Ordering::Less,
            turn,
        };
        turn.apply_and(self)
    }

    /// Search for any other turn / later turn from the same side
    pub fn not_max_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Not,
            order: Ordering::Less,
            turn,
        };
        turn.apply(self)
    }

    /// Or search for any other turn / later turn from the same side
    pub fn or_not_max_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Not,
            order: Ordering::Less,
            turn,
        };
        turn.apply_or(self)
    }

    /// And search for any other turn / later turn from the same side
    pub fn and_not_max_turn(self, turn: Turn) -> Self {
        let turn = TurnQueryParam {
            boolean: Bool::Not,
            order: Ordering::Less,
            turn,
        };
        turn.apply_and(self)
    }
}
