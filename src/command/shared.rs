use std::path::{Path, PathBuf};

use crate::{
    io_utils::read_input_from_user,
    save::{ParseSaveError, Save, TurnSave},
};

/// utility functions shared between commands

/// Return an iterator of parsed [`TurnSave`]s and their corresponding `PathBuf`.
///
/// Any autosaves are ignored.
///
/// it takes much care to return any io errors that may occur, so that they can be reported to the caller,
/// but any parsing errors are swallowed, since there could be other unrelated files in the same directory
pub(crate) fn iter_saves_in_dir<'a, 'b>(
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
                    Ok(Save::Turn(save)) => Some(Ok((save, path))),
                    Ok(Save::Autosave) => None,
                    Err(_) => None,
                }
            }
            Err(err) => Some(Err(err)),
        }))
}

pub(crate) fn get_confirmation(prompt: &str) -> std::io::Result<bool> {
    loop {
        let response = read_input_from_user(&format!("{prompt}: [Y] / N"))?;
        let response = response.trim();

        if response.is_empty() {
            // user pressed enter
            return Ok(true);
        }
        match response {
            "Y" | "y" => break Ok(true),
            "N" | "n" => break Ok(false),
            _ => {
                println!("Please confirm Y or N");
                continue;
            }
        }
    }
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
