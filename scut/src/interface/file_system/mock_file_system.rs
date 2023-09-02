use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::error::testing_error::MockError;

use super::FileSystem;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Status {
    Exists,
    Missing,
    Error,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    File(File),
    Folder(Folder),
}

/// Simulates a file
#[derive(Debug, PartialEq, Clone)]
pub struct File {
    pub path: PathBuf,
    pub status: Status,
    pub content: Option<String>,
}

impl File {
    pub fn new(path: PathBuf, status: Status, content: Option<String>) -> Self {
        File {
            path,
            status,
            content,
        }
    }
}

/// Simulates a folder
#[derive(Debug, PartialEq, Clone)]
pub struct Folder {
    pub path: PathBuf,
    pub status: Status,
    pub files: Vec<File>,
}

#[derive(Debug, PartialEq)]
pub struct MockFileSystem {
    objects: HashMap<PathBuf, Object>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Event {
    /// contains: path of the file, its status, whether it existed
    FileExists(PathBuf, Status, bool),

    /// contains: path of the folder, its status, len of its contents
    PathsInFolder(PathBuf, Status, usize),

    /// contains: path of the file, its status, what was written
    WriteStringToFile(PathBuf, Status, String),

    /// contains: path of the file, its status
    ReadFileToString(PathBuf, Status),

    /// contains: path of the relevant thing, a message
    TestFailure(PathBuf, String),
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FS EVENT: {self:?}")
    }
}

impl MockFileSystem {
    pub fn new() -> Self {
        MockFileSystem {
            objects: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, path: PathBuf, status: Status, content: Option<String>) {
        self.objects.insert(
            path.clone(),
            Object::File(File {
                path,
                status,
                content,
            }),
        );
    }

    pub fn add_folder(&mut self, path: PathBuf, status: Status) {
        self.objects.insert(
            path.clone(),
            Object::Folder(Folder {
                path,
                status,
                files: Vec::new(),
            }),
        );
    }

    pub fn add_file_to_folder(&mut self, folder: &Path, file: File) {
        if let Some(Object::Folder(folder)) = self.objects.get_mut(folder) {
            folder.files.push(file)
        }
    }

    pub fn add_file_inside_folder(&mut self, folder: &Path, file: File) {
        self.objects
            .insert(file.path.clone(), Object::File(file.clone()));

        match self.objects.get_mut(folder) {
            Some(Object::Folder(folder)) => {
                folder.files.push(file);
            }
            _ => panic!(
                "'{}' should be a folder in mock_filesystem: {self:?}",
                folder.display()
            ),
        }
    }

    pub fn set_file_content(&mut self, path: &Path, content: String) {
        if let Some(obj) = self.objects.get_mut(path) {
            match obj {
                Object::File(f) => f.content = Some(content),
                Object::Folder(_) => panic!(
                    "'{}' should be a file in mock_filesystem: {self:?}",
                    path.display()
                ),
            };
        }
    }

    pub fn get_file_content(&mut self, path: &Path) -> Option<&String> {
        if let Some(obj) = self.objects.get(path) {
            match obj {
                Object::File(f) => f.content.as_ref(),
                Object::Folder(_) => panic!(
                    "'{}' should be a file in mock_filesystem: {self:?}",
                    path.display()
                ),
            }
        } else {
            panic!(
                "'{}' should be a file in mock_filesystem: {self:?}",
                path.display()
            )
        }
    }
}

impl Default for MockFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a MockFileSystem from a line delimited list of paths
///
/// * Lines ending in `'/'` are added as folders
/// * Lines not ending in `'/'` are added as files
/// * Lines indented with 4 leading spaces are added to the previous folder and must be a relative path from that folder
/// * Empty lines "close" a folder if one is open and are otherwise ignored
/// * Folders cannot be nested!
/// * There is no support for '.' or ".." relative paths
///
/// # Missing and Error objects
/// * Objects prefixed with `!` will be added as an error - operations using that file/folder will raise an error
/// * Objects prefixed with `?` will be added as missing - operations using that file/folder will behave as if it didn't exist
/// * for lines indented with 4 leading spaces, the above object prefix must be used after the leading 4 spaces.
///
/// Note that intermediate folders do not need to be created, and the filesystem may be non-sensical!
///
/// For example:
/// ```text
/// /path/to/foo.rs
/// /path/to/folder/
///     inside_folder.txt
///     also_inside_folder.txt
///
/// /out/of/folder/again.rs
/// ```
impl FromStr for MockFileSystem {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut file_system = MockFileSystem::new();

        let mut current_folder: Option<PathBuf> = None;

        for line in s.lines() {
            if line.starts_with("!    ") || line.starts_with("?    ") {
                panic!("'{line}' should have the object prefix '!' or '?' come after the 4 leading spaces that indicate a nested file: {s:?}");
            }
            if line.starts_with("! ") || line.starts_with("? ") {
                panic!("'{line}' should have no space after the object prefix '!' or '?': {s:?}");
            }

            if let Some(indented_line) = line.strip_prefix("    ") {
                match indented_line.chars().last() {
                    Some(ch) => match ch {
                        '/' => panic!("'{line}' should not be a folder because MockFileSystem does not support nested folders: {s:?}"),
                        _ => if let Some(folder) = current_folder.as_ref() {
                            let file = match indented_line.chars().next() {
                                Some('!') => File::new(folder.join(&indented_line[1..]), Status::Error, None),
                                Some('?') => File::new(folder.join(&indented_line[1..]), Status::Missing, None),
                                Some(_) => File::new(folder.join(indented_line), Status::Exists, None),
                                None => unreachable!("line had a last char so it must have a first char: {s:?}")
                            };
                            file_system.add_file_inside_folder(folder, file)
                        } else {
                            panic!("'{line}' should come after a folder since it is indented: {s:?}")
                        }
                    },
                    None => panic!("'{line}' should contain a relative path because folders must contain files: {s:?}"),
                }
            } else {
                match line.chars().last() {
                    Some(ch) => match ch {
                        '/' => {
                            match line.chars().next() {
                                Some('!') => {
                                    file_system.add_folder(PathBuf::from(&line[1..]), Status::Error)
                                }
                                Some('?') => file_system
                                    .add_folder(PathBuf::from(&line[1..]), Status::Missing),
                                Some(_) => {
                                    file_system.add_folder(PathBuf::from(line), Status::Exists)
                                }
                                None => unreachable!(),
                            };
                            current_folder = Some(PathBuf::from(line))
                        }
                        _ => match line.chars().next() {
                            Some('!') => {
                                file_system.add_file(PathBuf::from(&line[1..]), Status::Error, None)
                            }
                            Some('?') => file_system.add_file(
                                PathBuf::from(&line[1..]),
                                Status::Missing,
                                None,
                            ),
                            Some(_) => {
                                file_system.add_file(PathBuf::from(line), Status::Exists, None)
                            }
                            None => unreachable!(),
                        },
                    },
                    None => {
                        current_folder = None;
                    }
                }
            }
        }

        Ok(file_system)
    }
}

impl FileSystem for MockFileSystem {
    fn file_exists(&mut self, path: &Path) -> anyhow::Result<bool> {
        match self.objects.get(path) {
            Some(Object::File(f)) => match f.status {
                status @ Status::Exists => {
                    let exists = true;

                    println!("{}", Event::FileExists(path.into(), status, exists));
                    Ok(exists)
                }
                status @ Status::Missing => {
                    let exists = false;

                    println!("{}", Event::FileExists(path.into(), status, exists));
                    Ok(exists)
                }
                status @ Status::Error => {
                    println!("{}", Event::FileExists(path.into(), status, false));
                    Err(MockError::new(status))?
                }
            },
            _ => panic!(
                "'{}' should be a file in mock_filesystem: {self:?}",
                path.display()
            ),
        }
    }

    fn paths_in_folder(&mut self, path: &Path) -> anyhow::Result<Vec<PathBuf>> {
        match self.objects.get(path) {
            Some(Object::Folder(folder)) => match folder.status {
                status @ Status::Exists => {
                    println!(
                        "{}",
                        Event::PathsInFolder(path.into(), status, folder.files.len(),)
                    );
                    folder
                        .files
                        .iter()
                        .cloned()
                        .filter_map(|f| match f.status {
                            Status::Exists => Some(Ok(f.path)),
                            Status::Missing => None,
                            Status::Error => {
                                Some(Err(anyhow::Error::from(MockError::new(Status::Error))))
                            }
                        })
                        .collect::<Result<Vec<_>, _>>()
                }
                status => {
                    println!(
                        "{}",
                        Event::PathsInFolder(path.into(), status, folder.files.len(),)
                    );
                    Err(MockError::new(status))?
                }
            },
            Some(Object::File(_)) => panic!(
                "'{}' should be a folder in mock filesystem: {self:?}",
                path.display()
            ),
            None => panic!(
                "'{}' should be a folder in mock filesystem: {self:?}",
                path.display()
            ),
        }
    }

    fn write_string_to_file(&mut self, content: &str, path: &Path) -> anyhow::Result<()> {
        match self.objects.get(path) {
            Some(Object::File(f)) => {
                let status = f.status;
                println!(
                    "{}",
                    Event::WriteStringToFile(path.into(), status, content.to_string(),)
                );
                match status {
                    Status::Exists => Ok(()),
                    status @ Status::Missing | status @ Status::Error => {
                        Err(MockError::new(status))?
                    }
                }
            }
            _ => panic!(
                "'{}' should be a file in mock filesystem: {self:?}",
                path.display()
            ),
        }
    }

    fn read_file_to_string(&mut self, path: &Path) -> anyhow::Result<String> {
        match self.objects.get(path) {
            Some(Object::File(f)) => {
                let status = f.status;
                println!("{}", Event::ReadFileToString(path.into(), status));
                match status {
                    Status::Exists => Ok(f.content.clone().unwrap_or_else(|| {
                        panic!(
                            "'{}' should be a file with content in mock filesystem: {self:?}",
                            path.display(),
                        )
                    })),
                    status @ Status::Missing | status @ Status::Error => {
                        Err(MockError::new(status))?
                    }
                }
            }
            _ => panic!(
                "'{}' should be a file in mock filesystem: {self:?}",
                path.display()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mock_file_system() {
        let string = r"
/path/to/foo.rs
/path/to/folder/
    inside_folder.txt
    also_inside_folder.txt

/out/of/folder/again.rs
";

        let mock_file_system: MockFileSystem = string.parse().unwrap();

        let foo = PathBuf::from("/path/to/foo.rs");
        assert_eq!(
            *mock_file_system.objects.get(&foo).unwrap(),
            Object::File(File {
                path: foo,
                status: Status::Exists,
                content: None
            })
        );

        let folder = PathBuf::from("/path/to/folder/");
        assert_eq!(
            *mock_file_system.objects.get(&folder).unwrap(),
            Object::Folder(Folder {
                path: folder,
                status: Status::Exists,
                files: vec![
                    File {
                        path: PathBuf::from("/path/to/folder/inside_folder.txt"),
                        status: Status::Exists,
                        content: None,
                    },
                    File {
                        path: PathBuf::from("/path/to/folder/also_inside_folder.txt"),
                        status: Status::Exists,
                        content: None,
                    }
                ],
            })
        );
    }

    #[test]
    fn test_parse_mock_file_system_folders_contain_files() {
        let string = r"
/path/to/folder/
    file_a
    file_b
";

        let mock_file_system: MockFileSystem = string.parse().unwrap();

        let folder = PathBuf::from("/path/to/folder/");
        let file_a = PathBuf::from("/path/to/folder/file_a");
        let file_b = PathBuf::from("/path/to/folder/file_b");

        assert_eq!(
            *mock_file_system.objects.get(&folder).unwrap(),
            Object::Folder(Folder {
                path: folder,
                status: Status::Exists,
                files: vec![
                    File {
                        path: file_a,
                        status: Status::Exists,
                        content: None,
                    },
                    File {
                        path: file_b,
                        status: Status::Exists,
                        content: None,
                    }
                ],
            })
        );
    }

    #[test]
    fn test_parse_mock_file_system_statuses() {
        let string = r"
!/path/to/error.rs
?/path/to/missing-folder/

/path/to/folder/
    !error_inside_folder.txt
    ?missing_file_inside_folder.txt
";

        let mock_file_system: MockFileSystem = string.parse().unwrap();

        let error = PathBuf::from("/path/to/error.rs");
        assert_eq!(
            *mock_file_system.objects.get(&error).unwrap(),
            Object::File(File {
                path: error,
                status: Status::Error,
                content: None
            })
        );

        let missing_folder = PathBuf::from("/path/to/missing-folder/");
        assert_eq!(
            *mock_file_system.objects.get(&missing_folder).unwrap(),
            Object::Folder(Folder {
                path: missing_folder,
                status: Status::Missing,
                files: Vec::new(),
            })
        );

        let error_inside_folder = PathBuf::from("/path/to/folder/error_inside_folder.txt");
        assert_eq!(
            *mock_file_system.objects.get(&error_inside_folder).unwrap(),
            Object::File(File {
                path: error_inside_folder,
                status: Status::Error,
                content: None,
            })
        );

        let missing_inside_folder = PathBuf::from("/path/to/folder/missing_file_inside_folder.txt");
        assert_eq!(
            *mock_file_system
                .objects
                .get(&missing_inside_folder)
                .unwrap(),
            Object::File(File {
                path: missing_inside_folder,
                status: Status::Missing,
                content: None,
            })
        );
    }
}
