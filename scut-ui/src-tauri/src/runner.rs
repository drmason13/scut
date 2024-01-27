use std::sync::{self, Mutex};

use scut_core::{
    interface::{
        predict::simple_predict::SimplePredict, LocalStorage, Predict, Prediction, RemoteStorage,
    },
    Config, Save,
};

use crate::{config::ready_config, storage::ready_storage};

pub struct ScutRunner {
    pub local: Box<dyn LocalStorage>,
    pub remote: Box<dyn RemoteStorage>,
    pub config: Config,
    pub predictor: SimplePredict,
}

impl ScutRunner {
    pub fn new() -> anyhow::Result<ScutRunner> {
        let (config, _) = ready_config(None)?;
        let (local, remote, config) = ready_storage(config)?;

        Ok(ScutRunner {
            local,
            remote,
            config,
            predictor: SimplePredict,
        })
    }

    pub fn make_prediction(mut self) -> anyhow::Result<Prediction> {
        let local = &mut *self.local;
        let remote = &mut *self.remote;

        self.predictor
            .predict(self.config.side, &self.config.player, None, local, remote)
    }

    pub fn upload(mut self, uploads: Vec<Save>) -> anyhow::Result<()> {
        let local = &mut *self.local;
        let remote = &mut *self.remote;

        let local_path = local.location();

        for save in uploads {
            remote.upload(&save, local_path)?;
        }

        Ok(())
    }

    pub fn download(mut self, downloads: Vec<Save>) -> anyhow::Result<()> {
        println!("downloading {downloads:?}");
        let local = &mut *self.local;
        let remote = &mut *self.remote;

        let local_path = local.location();

        for save in downloads {
            remote.download(&save, local_path)?;
        }

        Ok(())
    }
}

pub struct AppState {
    pub scut: sync::Mutex<ScutRunner>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        Ok(AppState {
            scut: Mutex::new(ScutRunner::new()?),
        })
    }
}
