#![cfg(test)]

use std::{fs::File, io::Write};

use tempfile::tempdir;

pub(crate) fn create_test_directory() -> tempfile::TempDir {
    // Create a directory inside of `std::env::temp_dir()`
    let dir = tempdir().expect("create test directory");

    let mut file =
        File::create(dir.path().join("Allies DM 70.sav")).expect("create test directory");
    writeln!(file, "This is a save file").expect("create test directory");

    std::thread::sleep(std::time::Duration::from_millis(10));

    let mut file = File::create(dir.path().join("test.dat")).expect("create test directory");
    writeln!(file, "This is a data file").expect("create test directory");

    std::thread::sleep(std::time::Duration::from_millis(10));

    let mut file = File::create(dir.path().join("autosave.sav")).expect("create test directory");
    writeln!(file, "This is a second save file").expect("create test directory");

    std::thread::sleep(std::time::Duration::from_millis(10));

    let mut file = File::create(dir.path().join("test2.dat")).expect("create test directory");
    writeln!(file, "This is a second data file").expect("create test directory");

    std::thread::sleep(std::time::Duration::from_millis(10));

    let mut file = File::create(dir.path().join("Allies DM 70.7z")).expect("create test directory");
    writeln!(file, "This is an archive file").expect("create test directory");

    std::thread::sleep(std::time::Duration::from_millis(10));

    let mut file = File::create(dir.path().join("Axis 70.7z")).expect("create test directory");
    writeln!(file, "This is a second archive file").expect("create test directory");

    dir
}
