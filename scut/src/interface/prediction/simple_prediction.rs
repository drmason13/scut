//! The names of these Prediction implementations are faily arbitrary!
//!
//! This Prediction implementation implements `predict_turn`, using the friendly turns already available in RemoteStorage to determine what turn it must be.

use crate::{
    interface::{index::Query, LocalStorage, RemoteStorage},
    Save, Side,
};

#[cfg(test)]
mod ddt;

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
        let enemy_turn = side.next_turn(turn);

        // for simplicity, always ask the user if they want to upload the autosave!
        // TODO: predict false if predict_downloads predicts any downloads
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
    use crate::interface::storage::mock_index_storage::MockIndexStorage;

    use super::*;

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
}
