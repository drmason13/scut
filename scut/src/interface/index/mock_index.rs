use std::slice::Iter;

use crate::Save;

use super::IterIndex;

/// Search a simple Vec of [`Save`]s
#[derive(Debug, Clone)]
pub struct MockIndex {
    saves: Vec<Save>,
}

impl MockIndex {
    pub fn new<'a, I>(saves: I) -> Self
    where
        I: IntoIterator<Item = &'a Save>,
    {
        let saves = saves.into_iter().cloned().collect();
        MockIndex { saves }
    }
}

impl<'a> IterIndex<'a> for MockIndex {
    type Iter = Iter<'a, Save>;

    fn iter(&'a self) -> Self::Iter {
        self.saves.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::interface::{index::Query, Index};

    use super::*;

    #[test]
    fn new_works() {
        use crate::{Save, Side};

        let saves = &[Save::new(Side::Allies, 1)];
        let _ = MockIndex::new(saves);
    }

    #[test]
    fn mock_index_is_searchable() {
        use crate::{Save, Side};

        let saves = &[Save::new(Side::Allies, 1)];
        let index = MockIndex::new(saves);

        assert_eq!(
            index.search(&Query::new()).unwrap(),
            vec![Save::new(Side::Allies, 1)]
        );
    }
}
