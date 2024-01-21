use super::{Query, QueryParam};

impl<'a> Query<'a> {
    /// Search for a particular part
    pub fn part(mut self, part: Option<&'a str>) -> Self {
        let part = QueryParam::is(part);
        self.part = Some(part);
        self
    }

    /// Or search for a particular part
    pub fn or_part(mut self, part: Option<&'a str>) -> Self {
        self.part = self.part.map(|p| p.or(part));
        self
    }

    /// Search for any other part
    pub fn not_part(mut self, part: Option<&'a str>) -> Self {
        let part = QueryParam::not(part);
        self.part = Some(part);
        self
    }

    /// Or search for a particular part
    pub fn or_not_part(mut self, part: Option<&'a str>) -> Self {
        self.part = self.part.map(|p| p.or_not(part));
        self
    }
}
