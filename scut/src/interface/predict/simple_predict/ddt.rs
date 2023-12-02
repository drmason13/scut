use std::path::{Path, PathBuf};

use parsely::{combinator::pad, token, until, ws, Lex, Parse};

use crate::{
    interface::{
        file_system::local_file_system::LocalFileSystem,
        storage::mock_index_storage::MockIndexStorage,
    },
    save::{parse_save, parse_side},
};

use super::*;
use pretty_assertions::assert_eq;

#[derive(Debug)]
pub struct TestCase {
    local: MockIndexStorage,
    remote: MockIndexStorage,
    side: Side,
    player: String,
    expected_autosave_prediction: AutosavePrediction,
    downloads_expected: Vec<Save>,
    uploads_expected: Vec<Save>,
}

impl TestCase {
    pub fn run(&mut self, idx: usize, pred: SimplePredict) -> anyhow::Result<()> {
        let turn = pred.predict_turn(
            self.side,
            self.player.as_str(),
            &mut self.local,
            &mut self.remote,
        )?;

        let actual_downloads = pred.predict_downloads(
            turn,
            self.side,
            self.player.as_str(),
            &mut self.local,
            &mut self.remote,
        )?;

        let actual_uploads = pred.predict_uploads(
            turn,
            self.side,
            self.player.as_str(),
            &mut self.local,
            &mut self.remote,
        )?;

        let actual_autosave_prediction = pred.predict_autosave(
            turn,
            &actual_downloads,
            self.side,
            self.player.as_str(),
            &mut self.local,
            &mut self.remote,
        )?;

        assert_eq!(
            self.expected_autosave_prediction, actual_autosave_prediction,
            "Predicted wrong autosave for test_case {idx}"
        );

        assert_eq!(
            self.downloads_expected, actual_downloads,
            "Predicted wrong downloads for test_case {idx}"
        );
        assert_eq!(
            self.uploads_expected, actual_uploads,
            "Predicted wrong uploads for test_case {idx}"
        );

        Ok(())
    }
}

/// ```text
/// <side> <turn>
///
/// <local saves>
/// [autosave = <save>[, true|false]]
///
/// <remote saves>
///
/// <downloads expected>
///
/// <uploads expected>
/// ```
pub fn parse_test_case(input: &str) -> Result<(TestCase, &str), parsely::Error> {
    let test_side_player_marker = pad(
        token("<"),
        token(">"),
        parse_side
            .then_skip(ws())
            .then(until(">").map(|s| String::from(s))),
    );
    let ((side, player), remaining) = test_side_player_marker
        .then_skip(ws().optional())
        .parse(input)?;

    let autosave_prediction_reason = token("AutosaveAlreadyUploaded")
        .map(|_| AutosavePredictionReason::AutosaveAlreadyUploaded)
        .or(token("TeammateSaveNotUploaded")
            .map(|_| AutosavePredictionReason::TeammateSaveNotUploaded))
        .or(token("NewTurnAvailable").map(|_| AutosavePredictionReason::NewTurnAvailable))
        .or(token("AutosaveNotAvailable").map(|_| AutosavePredictionReason::AutosaveNotAvailable));

    let comma = || token(",").then(ws().optional());

    let autosave_prediction = parse_save
        .then(comma().skip_then(autosave_prediction_reason))
        .map(|(save, reason)| AutosavePrediction::NotReady(save, reason))
        .or(parse_save
            .then_skip(comma().then(token("Ready")))
            .map(AutosavePrediction::Ready));

    let (expected_autosave_prediction, remaining) = token("Local:")
        .then(ws().many(..5))
        .then(token("autosave = "))
        .skip_then(autosave_prediction)
        .parse(remaining)?;

    let parse_saves = || {
        ws().optional()
            .skip_then(parse_save.then_skip(ws()).many(..9999))
            .then_skip(ws().optional())
    };

    let (local_saves, rem) = parse_saves().parse(remaining)?;
    let (remote_saves, rem) = token("Remote:").skip_then(parse_saves()).parse(rem)?;
    let (downloads_expected, rem) = token("Downloads:").skip_then(parse_saves()).parse(rem)?;
    let (uploads_expected, rem) = token("Uploads:").skip_then(parse_saves()).parse(rem)?;

    let local = MockIndexStorage::new(true, local_saves);
    let remote = MockIndexStorage::new(true, remote_saves);

    Ok((
        TestCase {
            expected_autosave_prediction,
            local,
            remote,
            side,
            player,
            downloads_expected,
            uploads_expected,
        },
        rem,
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

pub fn test_dir() -> PathBuf {
    PathBuf::from("./test_data/").join("simple_predict")
}

macro_rules! ddt {
    ($name:ident, $doc_comment:expr) => {
        paste::item! {
            #[doc = $doc_comment]
            #[test]
            fn [< simple_predict_ddt_ $name >]() -> anyhow::Result<()> {
                let $name = stringify!{ $name };
                let data_path = test_dir().join(format!("{}.txt", $name));

                let mut test_cases = read_test_cases(data_path.as_path())?;

                for (idx, test_case) in test_cases.iter_mut().enumerate() {
                    test_case.run(idx + 1, SimplePredict::default())?;
                }

                Ok(())
            }
        }
    };
}

ddt!(mixed, "both uploads and downloads (with/without autosave)");
ddt!(bugs, "Any fixed predict bugs get a test case");
ddt!(downloads, "should only predict downloads");
ddt!(uploads, "should only predict uploads");
ddt!(autosave_only, "should only predict autosave");
