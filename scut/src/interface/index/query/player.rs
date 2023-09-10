use std::default::Default;

use super::{builder::QueryBuildParameter, Bool, Query, SubQuery};

struct Player<'a>(Option<&'a str>);

impl<'a> QueryBuildParameter<'a> for Player<'a> {
    fn new_sub_query(self, boolean: bool) -> SubQuery<'a> {
        SubQuery {
            player: Some(if boolean {
                Bool::Is(self.0)
            } else {
                Bool::IsNot(self.0)
            }),
            ..Default::default()
        }
    }

    fn merge_into(self, mut sub_query: SubQuery<'a>, boolean: bool) -> SubQuery<'a> {
        sub_query.player = Some(if boolean {
            Bool::Is(self.0)
        } else {
            Bool::IsNot(self.0)
        });
        sub_query
    }
}

impl<'a> Query<'a> {
    /// Search for a particular player
    pub fn player(self, player: Option<&'a str>) -> Self {
        let player = Player(player);
        player.build(self)
    }

    /// Or search for a particular player
    pub fn or_player(self, player: Option<&'a str>) -> Self {
        let player = Player(player);
        player.build_or(self)
    }

    /// And search for a particular player
    pub fn and_player(self, player: Option<&'a str>) -> Self {
        let player = Player(player);
        player.build_and(self)
    }

    /// Search for any other player
    pub fn not_player(self, player: Option<&'a str>) -> Self {
        let player = Player(player);
        player.build_not(self)
    }

    /// Or search for any other player
    pub fn or_not_player(self, player: Option<&'a str>) -> Self {
        let player = Player(player);
        player.build_or_not(self)
    }

    /// And search for any other player
    pub fn and_not_player(self, player: Option<&'a str>) -> Self {
        let player = Player(player);
        player.build_and_not(self)
    }
}
