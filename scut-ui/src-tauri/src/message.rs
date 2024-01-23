//! Messages between the backend, window and tray menu

use scut_core::interface::Prediction;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Message {
    /// Trigger a run of "scut", to predict what files should be uploaded/downloaded
    ///
    /// Send to the backend
    Run,

    /// Contains the prediction from scut
    Prediction(Prediction),
}
