use std::{borrow::Borrow, fmt, io, path::PathBuf};

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
        // This is a bit hacky, but the output looks "nice"
        // for "OS errors" (an implementation detail of io::Error) the error message always ends with (os error *n*) where n is some error code
        // we'll insert the path before that, after the os description of the error code
        // for example
        // `No such file or directory (os error 2)`
        // becomes
        // `No such file or directory './path/to/nowhere' (os error 2)`
        let default_message = self.error.to_string();
        if let Some(index) = default_message.find("(os error ") {
            let safe_index = nightly::floor_char_boundary(&default_message, index);

            let mut default_message = default_message;
            let quoted_path = format!(" '{}'", self.path.to_string_lossy());
            default_message.insert_str(safe_index, &quoted_path);
            default_message.fmt(f)
        } else {
            // otherwise just append the path to the full error message, whatever it might look like!
            write!(f, "{}: '{}'", self.error, self.path.display())
        }
    }
}

mod nightly {
    // borrowed from: https://doc.rust-lang.org/src/core/str/mod.rs.html#254
    // #[unstable(feature = "round_char_boundary", issue = "93743")]
    #[inline]
    pub(super) fn floor_char_boundary(s: &str, index: usize) -> usize {
        if index >= s.len() {
            s.len()
        } else {
            let lower_bound = index.saturating_sub(3);
            let new_index = s.as_bytes()[lower_bound..=index]
                .iter()
                .rposition(|b| is_utf8_char_boundary(*b));

            // SAFETY: we know that the character boundary will be within four bytes
            unsafe { lower_bound + new_index.unwrap_unchecked() }
        }
    }

    // borrowed from: https://doc.rust-lang.org/src/core/num/mod.rs.html
    #[inline]
    const fn is_utf8_char_boundary(n: u8) -> bool {
        // This is bit magic equivalent to: b < 128 || b >= 192
        (n as i8) >= -0x40
    }
}
