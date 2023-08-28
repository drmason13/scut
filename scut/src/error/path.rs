use std::{error::Error, fmt, fmt::Write, io, path::PathBuf};

/// Extension trait to embed path information into relevant io::Errors
pub trait ErrorPaths<T> {
    fn path<P>(self, path: P) -> Result<T, ErrorWithPath>
    where
        P: Into<PathBuf>;
}

impl<T> ErrorPaths<T> for Result<T, io::Error> {
    fn path<P>(self, path: P) -> Result<T, ErrorWithPath>
    where
        P: Into<PathBuf>,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(ErrorWithPath::new(error, path.into())),
        }
    }
}

/// Wraps io::Error to add a related path and embeds that path within the io error's message
pub struct ErrorWithPath {
    pub error: io::Error,
    pub path: PathBuf,
}

impl ErrorWithPath {
    pub fn new(error: io::Error, path: PathBuf) -> Self {
        ErrorWithPath { error, path }
    }
}

impl fmt::Display for ErrorWithPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // This is a bit hacky, but the output looks "nice".
        // For "OS errors" (an implementation detail of io::Error) the error message always ends with (os error *n*) where n is some error code
        // We'll place that part with the path, which is typically usually more useful for the end user!
        // For example
        // `No such file or directory (os error 2)`
        // becomes
        // `No such file or directory './path/to/nowhere'`
        let default_message = self.error.to_string();
        if let Some(index) = default_message.find("(os error ") {
            let mut default_message = default_message;
            default_message.truncate(index - 1);
            write!(default_message, ": '{}'", self.path.display())?;
            default_message.fmt(f)
        } else {
            // otherwise just append the path to the full error message, whatever it might look like!
            write!(f, "{}: '{}'", self.error, self.path.display())
        }
    }
}

impl fmt::Debug for ErrorWithPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Error for ErrorWithPath {}
