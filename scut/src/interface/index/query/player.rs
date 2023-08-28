use super::{Query, QueryParam};

impl<'a> Query<'a> {
    /// Search for a particular player
    pub fn player(mut self, player: Option<&'a str>) -> Self {
        let player = QueryParam::is(player);
        self.player = Some(player);
        self
    }

    /// Or search for a particular player
    pub fn or_player(mut self, player: Option<&'a str>) -> Self {
        self.player = self.player.map(|p| p.or(player));
        self
    }

    /// Search for any other player
    pub fn not_player(mut self, player: Option<&'a str>) -> Self {
        let player = QueryParam::not(player);
        self.player = Some(player);
        self
    }

    /// Or search for any other player
    pub fn or_not_player(mut self, player: Option<&'a str>) -> Self {
        self.player = self.player.map(|p| p.or_not(player));
        self
    }
}
