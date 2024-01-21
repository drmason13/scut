use compose::Bool;

use crate::Turn;

use super::{turn_number::TurnNumberRange, Query, QueryParam};

impl<'a> Query<'a> {
    /// Search for a particular turn
    pub fn turn(mut self, turn: Turn) -> Self {
        self.turn_number = Some(QueryParam::is(TurnNumberRange::single(turn.number)));
        self.side = Some(QueryParam::is(turn.side));
        self
    }

    /// Or search for a particular turn
    pub fn or_turn(mut self, turn: Turn) -> Self {
        self.side = self.side.map(|s| s.or(turn.side));

        let turn_number = TurnNumberRange::single(turn.number);
        self.turn_number = self.turn_number.map(|t| t.or(turn_number));

        self
    }

    /// Search for any other turn
    pub fn not_turn(mut self, turn: Turn) -> Bool<Query<'a>> {
        self.turn_number = Some(QueryParam::is(TurnNumberRange::single(turn.number)));
        self.side = Some(QueryParam::is(turn.side));
        Bool::IsNot(self)
    }

    /// Or search for any other turn
    pub fn or_not_turn(mut self, turn: Turn) -> Self {
        self.side = self.side.map(|s| s.or(turn.side));

        let turn_number = TurnNumberRange::single(turn.number);
        self.turn_number = self.turn_number.map(|t| t.or_not(turn_number));

        self
    }

    /// Search for a particular turn, or a later turn from the same side
    pub fn min_turn(mut self, turn: Turn) -> Self {
        self.turn_number = Some(QueryParam::is(TurnNumberRange::from_start_end(
            Some(turn.number),
            None,
        )));
        self.side = Some(QueryParam::is(turn.side));
        self
    }

    /// Or search for a particular turn, or a later turn from the same side
    pub fn or_min_turn(mut self, turn: Turn) -> Self {
        self.side = self.side.map(|s| s.or(turn.side));

        let turn_number = TurnNumberRange::from_start_end(Some(turn.number), None);
        self.turn_number = self.turn_number.map(|t| t.or(turn_number));

        self
    }

    /// Search for a particular turn, or an earlier turn from the same side
    pub fn max_turn(mut self, turn: Turn) -> Self {
        self.turn_number = Some(QueryParam::is(TurnNumberRange::from_start_end(
            None,
            Some(turn.number),
        )));
        self.side = Some(QueryParam::is(turn.side));
        self
    }

    /// Or search for a particular turn, or an earlier turn from the same side
    pub fn or_max_turn(mut self, turn: Turn) -> Self {
        self.side = self.side.map(|s| s.or(turn.side));

        let turn_number = TurnNumberRange::from_start_end(None, Some(turn.number));
        self.turn_number = self.turn_number.map(|t| t.or(turn_number));

        self
    }
}
