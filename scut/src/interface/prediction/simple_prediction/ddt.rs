use std::path::{Path, PathBuf};

use parsely::{combinator::pad, token, until, ws, Lex, Parse};

use crate::{
    interface::{
        file_system::local_file_system::LocalFileSystem,
        storage::mock_index_storage::MockIndexStorage,
    },
    save::{parse_save, parse_side},
    SaveOrAutosave,
};

use super::*;
use pretty_assertions::assert_eq;

#[derive(Debug)]
pub struct TestCase {
    local: MockIndexStorage,
    remote: MockIndexStorage,
    side: Side,
    player: String,
    autosave_expected: Option<(Save, Option<bool>)>,
    downloads_expected: Vec<Save>,
    uploads_expected: Vec<Save>,
}

impl TestCase {
    pub fn run(&mut self, idx: usize, pred: SimplePrediction) -> anyhow::Result<()> {
        let turn = pred
            .predict_turn(
                self.side,
                self.player.as_str(),
                &mut self.local,
                &mut self.remote,
            )?
            .expect("turn should be predicted");

        let actual_autosave = pred.predict_autosave(
            turn,
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

        if let Some(autosave_expected) = self.autosave_expected.as_ref() {
            assert_eq!(
                *autosave_expected, actual_autosave,
                "Predicted wrong autosave for test_case {idx}"
            );
        }
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

    let bool = token("true")
        .map(|_| true)
        .or(token("false").map(|_| false));

    let autosave = token("autosave").map(|_| SaveOrAutosave::Autosave);
    let autosave_expected = token(",").then(ws().optional()).skip_then(bool);
    let save_and_autosave_expected = parse_save.then(autosave_expected.optional());

    let ((autosave, autosave_expected), remaining) = token("Local:")
        .then(ws().many(..5))
        .skip_then(
            autosave
                .optional()
                .then_skip(token(" = "))
                .then(save_and_autosave_expected.optional()),
        )
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

    let has_autosave_file = autosave.is_some();
    let local = MockIndexStorage::new(has_autosave_file, local_saves);
    let remote = MockIndexStorage::new(has_autosave_file, remote_saves);

    Ok((
        TestCase {
            autosave_expected,
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
    PathBuf::from("./test_data/").join("simple_prediction")
}

macro_rules! ddt_test {
    ($name:ident, $doc_comment:expr) => {
        paste::item! {
            #[doc = $doc_comment]
            #[test]
            fn [< simple_prediction_ddt_ $name >]() -> anyhow::Result<()> {
                let $name = stringify!{ $name };
                let data_path = test_dir().join(format!("{}.txt", $name));

                let mut test_cases = read_test_cases(data_path.as_path())?;

                for (idx, test_case) in test_cases.iter_mut().enumerate() {
                    test_case.run(idx + 1, SimplePrediction)?;
                }

                Ok(())
            }
        }
    };
}

ddt_test!(
    mixed,
    "test cases where prediction predicts both uploads and downloads (with/without autosave)"
);
ddt_test!(
    bugs,
    "Any fixed prediction bugs can have their own test case"
);
ddt_test!(
    downloads,
    "test cases where prediction should only predict downloads"
);
ddt_test!(
    uploads,
    "test cases where prediction should only predict uploads (with/without autosave)"
);
ddt_test!(
    autosave_only,
    "test cases where prediction should only predict autosave true/false (no downloads or uploads)"
);
