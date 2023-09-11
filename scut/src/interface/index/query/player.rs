use std::default::Default;

use super::{builder::QueryParam, Bool, Query, SubQuery};

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerQueryParam<'a> {
    boolean: Bool,
    player: Option<&'a str>,
}

impl<'a> QueryParam<'a> for PlayerQueryParam<'a> {
    type Value = Option<&'a str>;

    fn matches(&self, value: Self::Value) -> bool {
        self.player == value
    }

    fn new_sub_query(self) -> SubQuery<'a> {
        SubQuery {
            player: Some(self),
            ..Default::default()
        }
    }

    fn merge_into(self, mut sub_query: SubQuery<'a>) -> SubQuery<'a> {
        sub_query.player = Some(self);
        sub_query
    }
}

impl<'a> Query<'a> {
    /// Search for a particular player
    pub fn player(self, player: Option<&'a str>) -> Self {
        let player = PlayerQueryParam {
            boolean: Bool::Is,
            player,
        };
        player.apply(self)
    }

    /// Or search for a particular player
    pub fn or_player(self, player: Option<&'a str>) -> Self {
        let player = PlayerQueryParam {
            boolean: Bool::Is,
            player,
        };
        player.apply_or(self)
    }

    /// And search for a particular player
    pub fn and_player(self, player: Option<&'a str>) -> Self {
        let player = PlayerQueryParam {
            boolean: Bool::Is,
            player,
        };
        player.apply_and(self)
    }

    /// Search for any other player
    pub fn not_player(self, player: Option<&'a str>) -> Self {
        let player = PlayerQueryParam {
            boolean: Bool::Not,
            player,
        };
        player.apply(self)
    }

    /// Or search for any other player
    pub fn or_not_player(self, player: Option<&'a str>) -> Self {
        let player = PlayerQueryParam {
            boolean: Bool::Not,
            player,
        };
        player.apply_or(self)
    }

    /// And search for any other player
    pub fn and_not_player(self, player: Option<&'a str>) -> Self {
        let player = PlayerQueryParam {
            boolean: Bool::Not,
            player,
        };
        player.apply_and(self)
    }
}
