//! utility functions shared between commands

use std::path::{Path, PathBuf};

use crate::{
    config::Config,
    io_utils::iter_files_with_extension,
    save::{ParseSaveError, SavOrArchive, Save, TurnSave},
    side::Side,
};

/// find a save in `dir` belonging to the given `player` and `side` for the given `turn`
pub(crate) fn find_turn_start_save(
    dir: &Path,
    side: Side,
    turn: u32,
) -> std::io::Result<Option<(TurnSave, PathBuf)>> {
    let saves = iter_turn_saves_in_dir(dir, SavOrArchive::Archive.extension())?;

    Ok(saves
        .filter_map(|result| result.ok())
        .find(|(save, _)| save.turn == turn && save.side == side && save.player.is_none()))
}

/// find a save in `dir` belonging to the given `player` and `side` for the given `turn`
pub(crate) fn find_save(
    dir: &Path,
    side: Side,
    player: &String,
    turn: u32,
    file_type: SavOrArchive,
) -> std::io::Result<Option<(TurnSave, PathBuf)>> {
    let saves = iter_turn_saves_in_dir(dir, file_type.extension())?;

    Ok(saves.filter_map(|result| result.ok()).find(|(save, _)| {
        save.turn == turn && save.side == side && save.player.as_ref() == Some(player)
    }))
}

/// find a save in `dir` from a teammate of the given `player`, `side` for the given `turn`
pub(crate) fn find_team_save(
    dir: &Path,
    side: Side,
    player: &String,
    turn: u32,
    file_type: SavOrArchive,
) -> std::io::Result<Option<(TurnSave, PathBuf)>> {
    let saves = iter_turn_saves_in_dir(dir, file_type.extension())?;

    Ok(saves.filter_map(|result| result.ok()).find(|(save, _)| {
        save.turn == turn
            && save.side == side
            && save.player.is_some()
            && save.player.as_ref() != Some(player)
    }))
}

pub(crate) fn check_for_team_save(
    config: &Config,
    turn: u32,
    file_type: SavOrArchive,
) -> std::io::Result<bool> {
    let saves = find_team_save(&config.saves, config.side, &config.player, turn, file_type)?;

    Ok(saves.is_some())
}

/// find the autosave
pub(crate) fn find_autosave(dir: &Path) -> std::io::Result<Option<(Save, PathBuf)>> {
    let saves = iter_saves_in_dir(dir, "sav")?;

    Ok(saves
        .filter_map(|result| result.ok())
        .find(|(save, _)| save.is_autosave()))
}

/// Return an iterator of parsed [`TurnSave`]s and their corresponding `PathBuf`.
///
/// Any autosaves are ignored.
///
/// it takes much care to return any io errors that may occur, so that they can be reported to the caller,
/// but any parsing errors are swallowed, since there could be other unrelated files in the same directory
fn iter_saves_in_dir<'a, 'b>(
    dir: &'a Path,
    extension: &'b str,
) -> std::io::Result<impl Iterator<Item = std::io::Result<(Save, PathBuf)>> + 'a>
where
    'b: 'a,
{
    Ok(std::fs::read_dir(dir)?
        .map(|entry| entry.map(|e| e.path()))
        .filter(move |path| match path {
            Ok(path) => matches!(path.extension(), Some(ext) if ext == extension),
            Err(_) => true, // pass thru io errors
        })
        .filter_map(|path| match path {
            Ok(path) => {
                let save: Result<Save, ParseSaveError> = (&path).try_into();
                match save {
                    Ok(save) => Some(Ok((save, path))),
                    Err(_) => None,
                }
            }
            Err(err) => Some(Err(err)),
        }))
}

/// Return an iterator of parsed [`TurnSave`]s and their corresponding `PathBuf`.
///
/// Any autosaves are ignored.
///
/// it takes much care to return any io errors that may occur, so that they can be reported to the caller,
/// but any parsing errors are swallowed, since there could be other unrelated files in the same directory
pub(crate) fn iter_turn_saves_in_dir<'a, 'b>(
    dir: &'a Path,
    extension: &'b str,
) -> std::io::Result<impl Iterator<Item = std::io::Result<(TurnSave, PathBuf)>> + 'a>
where
    'b: 'a,
{
    Ok(
        iter_files_with_extension(dir, extension)?.filter_map(|path| match path {
            Ok(path) => {
                let save: Result<Save, ParseSaveError> = (&path).try_into();
                match save {
                    Ok(Save::Turn(save)) => Some(Ok((save, path))),
                    Ok(Save::Autosave) => None,
                    Err(_) => None,
                }
            }
            Err(err) => Some(Err(err)),
        }),
    )
}

#[cfg(test)]
mod test {
    use crate::side::Side;
    use crate::test::create_test_directory;

    use super::*;

    #[test]
    fn test_iter_turn_saves_in_dir() {
        let test_dir = create_test_directory();

        let mut saves = iter_turn_saves_in_dir(test_dir.path(), "7z")
            .expect("could not list entries in test dir")
            .filter_map(|save| save.ok());

        assert!(
            saves.any(|(save, _)| save.turn == 70
                && save.player == Some(String::from("DM"))
                && save.side == Side::Allies),
            "Must find the archive file like 'Allies DM 70.7z'"
        );

        let mut saves = iter_turn_saves_in_dir(test_dir.path(), "sav")
            .expect("could not list entries in test dir")
            .filter_map(|save| save.ok());

        assert!(
            saves.any(|(save, _)| save.turn == 70
                && save.player == Some(String::from("DM"))
                && save.side == Side::Allies),
            "Must find the save file like 'Allies DM 70.sav'"
        );

        let mut saves = iter_turn_saves_in_dir(test_dir.path(), "sav")
            .expect("could not list entries in test dir")
            .filter_map(|save| save.ok());

        assert!(
            !saves.any(|(save, _)| save.turn == 999
                && save.player == Some(String::from("nobody"))
                && save.side == Side::Allies),
            "Must not find save files that don't exist"
        );
    }
}
