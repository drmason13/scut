pub mod classic_predict;
pub mod simple_predict;

use crate::{Save, Side, Turn};

use super::{LocalStorage, RemoteStorage};

#[derive(Debug)]
pub struct Prediction {
    pub autosave: AutosavePrediction,
    pub uploads: Vec<Save>,
    pub downloads: Vec<Save>,
}

/// The Save that an autosave would be uploaded as, wrapped in an indication of whether it is ready to upload
#[derive(Debug, PartialEq, Eq)]
pub enum AutosavePrediction {
    /// Autosave is ready
    Ready(Save),
    NotReady(Save, AutosavePredictionReason),
}

/// Describes why an autosave might not be willing or able to upload the autosave
#[derive(Debug, PartialEq, Eq)]
pub enum AutosavePredictionReason {
    /// If the autosave is already there, you might not want to upload it again, because doing so will overwrite it
    AutosaveAlreadyUploaded,
    /// When no save from your teammate is in the remote for this turn, it means they haven't played their part of the turn yet
    TeammateSaveNotUploaded,
    /// When you have predicted downloads, that means your teammate has played a turn you haven't seen yet
    NewTurnAvailable,
    /// If the autosave isn't in local storage, you **cannot** upload it
    AutosaveNotAvailable,
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
        turn_override: Option<u32>,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Prediction>;

    /// Predict what turn it is.
    fn predict_turn(
        &self,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Turn>;

    /// Return all the [`Save`]s that should be downloaded.
    fn predict_downloads(
        &self,
        predicted_turn: Turn,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<Save>>;

    /// Return all the [`Save`]s that should be uploaded - disregarding the autosave, which is handled via [`predict_autosave`](Predict::predict_autosave)
    fn predict_uploads(
        &self,
        predicted_turn: Turn,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<Save>>;

    /// Predict what to upload the autosave as, if at all
    fn predict_autosave(
        &self,
        predicted_turn: Turn,
        predicted_downloads: &[Save],
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<AutosavePrediction>;
}
