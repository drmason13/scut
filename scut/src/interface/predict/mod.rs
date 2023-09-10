pub mod classic_predict;
pub mod simple_predict;

use crate::{Save, Side, Turn};

use super::{LocalStorage, RemoteStorage};

#[derive(Debug)]
pub struct Prediction {
    autosave: Option<Save>,
    uploads: Vec<Save>,
    downloads: Vec<Save>,
}

#[derive(Debug)]
pub struct TurnDetail {
    /// The latest turn you have uploaded belonging to your side
    ///
    /// 0 if no turn has been uploaded
    pub your_turn: Turn,

    /// The latest turn your teammate has uploaded belonging to your side
    ///
    /// 0 if no turn has been uploaded
    pub teammate_turn: Turn,

    /// The turn you want to start next
    ///
    /// This has a minimum of 1, nobody wants to start turn 0
    pub next_friendly_turn: Turn,

    /// The turn the enemy wants to start next
    ///
    /// This has a minimum of 1, nobody wants to start turn 0
    pub next_enemy_turn: Turn,
}

/// This trait is for the logic behind choosing which saves to download, which saves to upload and what turn the autosave should be uploaded as.
///
/// [`predict_downloads`]: Predict::predict_downloads
/// [`predict_uploads`]: Predict::predict_uploads
/// [`predict_turn`]: Predict::predict_turn
/// [`predict_autosave`]: Predict::predict_autosave
pub trait Predict {
    /// Predict what saves should be uploaded and downloaded
    fn predict(
        &self,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Prediction>;

    /// Predict what turn it is in detail. This is more complicated than you might expect.
    fn predict_turn_detail(
        &self,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<TurnDetail>;

    /// Return all the [`Save`]s that should be downloaded.
    fn predict_downloads(
        &self,
        turn_detail: &TurnDetail,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<Save>>;

    /// Return all the [`Save`]s that should be uploaded - disregarding the autosave, which is handled via [`predict_autosave`](Predict::predict_autosave)
    fn predict_uploads(
        &self,
        turn_detail: &TurnDetail,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<Save>>;

    /// Predict what to upload the autosave as, if at all
    fn predict_autosave(
        &self,
        turn_detail: &TurnDetail,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Option<Save>>;
}
