pub mod classic_predict;
pub mod simple_predict;

use serde::{Deserialize, Serialize};

use crate::{Save, Side, Turn};

use super::{LocalStorage, RemoteStorage};

/// Scut's prediction of what saves should be uploaded/downloaded
///
/// The autosave is always included in the prediction,
/// with a reason provided if it should not be uploaded.
///
/// The autosave prediction indicates what scut thinks the next enemy turn is.
/// The reason dictates whether scut thinks the autosave should be uploaded.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prediction {
    /// Whether scut suggests uploading the autosave
    ///
    /// (the user may choose to override)
    pub autosave: AutosavePrediction,

    /// What saves scut suggests to upload
    pub uploads: Vec<Save>,

    /// What saves suggests to download
    pub downloads: Vec<Save>,
}

/// The Save that an autosave would be uploaded as, wrapped in an indication of whether it is ready to upload
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "save")]
#[serde(rename = "camelCase")]
/*
   {
       "status": "ready",
       "save": { ... }
   }

   {
       "status": "notReady",
       "save": [{ ... }, "autosaveAlreadyUploaded"]
   }
*/
pub enum AutosavePrediction {
    /// Autosave is ready
    Ready(Save),
    NotReady(Save, AutosavePredictionReason),
}

/// Describes why an autosave might not be willing or able to upload the autosave
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
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
