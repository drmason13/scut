//! The names of these Prediction implementations are faily arbitrary!
//!
//! This Prediction implementation implements `predict_turn`, using the enemy turns available in RemoteStorage to determine what turn it must be...
//! Waiting *patiently* for your teammate to upload a turn start save for the enemy turn

use crate::{
    interface::{index::Query, LocalStorage, RemoteStorage},
    Save, Side,
};

use super::Prediction;

pub struct SimplePrediction;

impl Prediction for SimplePrediction {
    fn predict_turn(
        &self,
        friendly_side: Side,
        _player: &str,
        _local_storage: &mut dyn LocalStorage,
        remote_storage: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Option<u32>> {
        let remote_index = remote_storage.index();
        let turn = if let Some(save) = remote_index.latest_save(friendly_side)? {
            save.turn
        } else {
            1
        };

        Ok(Some(turn))
    }

    fn predict_downloads(
        &self,
        turn: u32,
        side: Side,
        _player: &str,
        local_storage: &mut dyn LocalStorage,
        remote_storage: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<crate::Save>> {
        let query = Query::new()
            .side(side)
            .turn_in_range(Some(turn.saturating_sub(1)), None);

        let local_saves = local_storage.index().search(&query)?;
        let remote_saves = remote_storage.index().search(&query)?;

        let missing_local_saves = remote_saves
            .into_iter()
            .filter(|s| !local_saves.contains(s))
            .collect();

        Ok(missing_local_saves)
    }

    fn predict_uploads(
        &self,
        turn: u32,
        side: Side,
        _player: &str,
        local_storage: &mut dyn LocalStorage,
        remote_storage: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<crate::Save>> {
        let query = Query::new()
            .side(side)
            .turn_in_range(Some(turn.saturating_sub(1)), None);

        let local_saves = local_storage.index().search(&query)?;
        let remote_saves = remote_storage.index().search(&query)?;

        let missing_friendly_saves = local_saves
            .into_iter()
            .filter(|s| !remote_saves.contains(s))
            .collect();

        Ok(missing_friendly_saves)
    }

    fn predict_autosave(
        &self,
        turn: u32,
        side: Side,
        _player: &str,
        _local_storage: &mut dyn LocalStorage,
        _remote_storage: &mut dyn RemoteStorage,
    ) -> anyhow::Result<(crate::Save, Option<bool>)> {
        let enemy_side = side.other_side();
        let enemy_turn = match enemy_side {
            // Allies play the same turn as Axis (Axis *1*, Allies *1*, Axis 2, Allies 2, ...)
            Side::Allies => turn,
            // Axis play the turn after (Axis 1, Allies *1*, Axis *2*, Allies 2, ...)
            Side::Axis => turn + 1,
        };

        // for simplicity, always ask the user if they want to upload the autosave!
        Ok((
            Save {
                turn: enemy_turn,
                side: enemy_side,
                player: None,
                part: None,
            },
            None,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use parsely::{token, until, ws, Lex, Parse};

    use crate::{
        interface::{
            file_system::local_file_system::LocalFileSystem,
            storage::mock_index_storage::MockIndexStorage,
        },
        save::parse_save,
    };

    use super::*;

    pub struct TestCase {
        local: MockIndexStorage,
        remote: MockIndexStorage,
        side: Side,
        player: String,
        downloads_expected: Vec<Save>,
        uploads_expected: Vec<Save>,
    }

    impl TestCase {
        pub fn run(&mut self, pred: SimplePrediction) -> anyhow::Result<()> {
            let turn = pred.predict_turn(
                self.side,
                self.player.as_str(),
                &mut self.local,
                &mut self.remote,
            )?;

            let actual_downloads = pred.predict_downloads(
                turn.expect("turn should be predicted"),
                self.side,
                self.player.as_str(),
                &mut self.local,
                &mut self.remote,
            )?;

            let actual_uploads = pred.predict_uploads(
                turn.expect("turn should be predicted"),
                self.side,
                self.player.as_str(),
                &mut self.local,
                &mut self.remote,
            )?;

            assert_eq!(self.downloads_expected, actual_downloads);
            assert_eq!(self.uploads_expected, actual_uploads);

            Ok(())
        }
    }

    /// ```text
    /// <side> <turn>
    ///
    /// <local saves>
    ///
    /// <remote saves>
    ///
    /// <downloads expected>
    ///
    /// <uploads expected>
    /// ```
    pub fn parse_test_case(input: &str) -> Result<(TestCase, &str), parsely::Error> {
        let ((side, player), remaining) = token("<")
            .skip_then(
                (token("Axis").map(|_| Side::Axis)).or(token("Allies").map(|_| Side::Allies)),
            )
            .then_skip(ws())
            .then(until(">").map(|s| String::from(s)))
            .then_skip(token(">").then(ws().many(..).then(token("Local:"))))
            .parse(input)?;

        let (tmp, remaining) = until("\n\n").lex(remaining)?;

        let (local_saves, _) = parse_save.pad().many(..9999).parse(tmp)?;

        let (_, remaining) = token("\n\nRemote:").lex(remaining)?;
        let (tmp, remaining) = until("\n\n").lex(remaining)?;
        let (remote_saves, _) = parse_save.pad().many(..9999).parse(tmp)?;

        let (_, remaining) = token("\n\nDownloads:").lex(remaining)?;
        let (tmp, remaining) = until("\n\n").lex(remaining)?;
        let (downloads_expected, _) = parse_save.pad().many(..9999).parse(tmp)?;

        let (_, remaining) = token("\n\nUploads:").lex(remaining)?;
        let (tmp, remaining) = until("\n\n").lex(remaining)?;
        let (uploads_expected, _) = parse_save.pad().many(..9999).parse(tmp)?;

        let local = MockIndexStorage::new(local_saves);
        let remote = MockIndexStorage::new(remote_saves);

        Ok((
            TestCase {
                local,
                remote,
                side,
                player,
                downloads_expected,
                uploads_expected,
            },
            remaining,
        ))
    }

    pub fn read_test_cases(data_path: &Path) -> anyhow::Result<Vec<TestCase>> {
        use crate::interface::FileSystem;

        let mut lfs = LocalFileSystem;

        let content = lfs.read_file_to_string(data_path)?;

        let (test_cases, _) = parse_test_case
            .pad()
            .many(1..9999)
            .parse(content.as_str())?;

        Ok(test_cases)
    }

    #[test]
    fn simple_prediction_works() -> anyhow::Result<()> {
        let prediction = SimplePrediction;

        let mut remote_storage = MockIndexStorage::new(vec![
            Save::new(Side::Axis, 1),
            Save::new(Side::Axis, 1).player("DM"),
            Save::new(Side::Axis, 1).player("DG"),
            Save::new(Side::Axis, 2),
            Save::new(Side::Allies, 1),
            Save::new(Side::Allies, 1).player("GM"),
            Save::new(Side::Allies, 1).player("TG"),
        ]);

        let mut local_storage = MockIndexStorage::new(vec![
            Save::new(Side::Axis, 1),
            Save::new(Side::Axis, 1).player("DM"),
        ]);

        assert_eq!(
            prediction.predict_downloads(
                1,
                Side::Axis,
                "DG",
                &mut local_storage,
                &mut remote_storage
            )?,
            vec![
                Save::new(Side::Axis, 1).player("DG"),
                Save::new(Side::Axis, 2),
            ]
        );

        Ok(())
    }

    #[test]
    fn simple_production_ddt() -> anyhow::Result<()> {
        let data_path = PathBuf::from("./test_data.txt");

        let mut test_cases = read_test_cases(data_path.as_path())?;

        for test_case in test_cases.iter_mut() {
            test_case.run(SimplePrediction)?;
        }

        Ok(())
    }
}
