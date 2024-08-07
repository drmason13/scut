use anyhow::{bail, Context};
use scut_core::{
    interface::{
        config::ConfigService, predict::simple_predict::SimplePredict, LocalStorage, Predict,
        Prediction, RemoteStorage,
    },
    Config, Save,
};

use crate::{config::ready_config, storage::ready_storage};

pub struct ScutRunner {
    pub local: Box<dyn LocalStorage>,
    pub remote: Box<dyn RemoteStorage>,
    pub config: Config,
    pub config_service: Box<dyn ConfigService>,
    pub predictor: SimplePredict,
}

impl ScutRunner {
    pub fn new() -> anyhow::Result<ScutRunner> {
        let (config, config_service) = ready_config(None)?;
        let (local, remote, config) = ready_storage(config)?;
        let predictor = SimplePredict::default();

        Ok(ScutRunner {
            local,
            remote,
            config,
            config_service,
            predictor,
        })
    }

    pub fn make_prediction(mut self) -> anyhow::Result<Prediction> {
        let local = &mut *self.local;
        let remote = &mut *self.remote;

        self.predictor.predict(
            self.config.side,
            &self.config.player,
            None,
            self.config.solo.unwrap_or_default(),
            local,
            remote,
        )
    }

    pub fn upload(mut self, autosave: Option<Save>, uploads: Vec<Save>) -> anyhow::Result<()> {
        let local = &mut *self.local;
        let remote = &mut *self.remote;

        if let Some(save) = autosave {
            let local_path = local
                .locate_autosave()?
                .expect("scut predicted need to upload autosave, so it must exist");
            remote.upload(&save, local_path.as_path())?;
        }

        for save in uploads {
            let local_path = local.locate_save(&save)?.ok_or_else(|| {
                anyhow::anyhow!(
                    "scut predicted the need to upload \
                your save '{}', but the corresponding file was not found!",
                    &save
                )
            })?;
            remote.upload(&save, local_path.as_path())?;
        }

        Ok(())
    }

    pub fn download(mut self, downloads: Vec<Save>) -> anyhow::Result<()> {
        let local = &mut *self.local;
        let remote = &mut *self.remote;

        let local_path = local.location();

        for save in downloads {
            remote.download(&save, local_path)?;
        }

        Ok(())
    }

    pub fn config(mut self) -> anyhow::Result<()> {
        let new_string = match edit::edit(self.config_service.serialize(&self.config)?) {
            Ok(new_string) => new_string,
            Err(io_err) if io_err.kind() == std::io::ErrorKind::InvalidData => {
                bail!("Unable to edit config with non-UTF8 content!");
            }
            Err(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
                bail!("Unable to find an editor to edit the config\n
                          You can edit the config from the commandline using `scut config set KEY VALUE`");
            }
            Err(e) => return Err(e).context("failed to open an editor to edit the config"),
        };

        let new_config = match self.config_service.deserialize(new_string.as_str()) {
            Ok(config) => config,
            Err(e) => {
                bail!("Invalid config: {e}\nYour changes have not been saved.");
            }
        };

        self.config_service
            .save(&new_config)
            .context("failed to save changes to config")?;

        Ok(())
    }
}
