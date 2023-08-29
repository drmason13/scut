//! The names of these Prediction implementations are faily arbitrary!
//!
//! This Prediction implementation implements `predict_turn`, using the enemy turns available in RemoteStorage to determine what turn it must be...
//! Waiting *patiently* for your teammate to upload a turn start save for the enemy turn

use crate::interface::{LocalStorage, UserInteraction};

use super::Prediction;

pub struct PatientPrediction {
    local_storage: Box<dyn LocalStorage>,
    user_interaction: Box<dyn UserInteraction>,
}

impl Prediction for PatientPrediction {
    fn predict_downloads(&self, turn: u32) -> Vec<crate::Save> {
        todo!()
    }

    fn predict_uploads(&self, turn: u32) -> Vec<crate::Save> {
        todo!()
    }

    fn predict_autosave(&self, turn: u32) -> (crate::Save, bool) {
        todo!()
    }

    fn predict_turn(&self) -> Option<u32> {
        None
    }
}
