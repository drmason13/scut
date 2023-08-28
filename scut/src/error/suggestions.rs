use std::{error::Error, fmt};

/// Wraps anyhow::Error to add a related "Suggestion", which is a const str advising the user how to proceed
pub struct ErrorWithSuggestion {
    pub error: anyhow::Error,
    pub suggestion: &'static str,
}

pub trait ErrorSuggestions<T> {
    /// Add a suggestion to an [`Error`](std::error::Error) type, returning an [`anyhow::Error`] containing an [`ErrorWithSuggestion`]
    fn suggest(self, suggestion: &'static str) -> Result<T, anyhow::Error>;
}

impl<T, E> ErrorSuggestions<T> for Result<T, E>
where
    anyhow::Error: From<E>,
    anyhow::Error: From<ErrorWithSuggestion>,
{
    fn suggest(self, suggestion: &'static str) -> Result<T, anyhow::Error> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(anyhow::Error::from(ErrorWithSuggestion {
                error: anyhow::Error::from(err),
                suggestion,
            })),
        }
    }
}

/// Our Display implementation forwards to the wrapped [`anyhow::Error`], ignoring the suggestion
///
/// The suggestion is printed in the verbose Debug implentation
impl fmt::Display for ErrorWithSuggestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}

impl fmt::Debug for ErrorWithSuggestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)?;
        write!(f, "\n\n")?;
        writeln!(f, "> {}", self.suggestion)
    }
}

/// Implement [`Error`](std::error::Error) in order to support conversion into [`anyhow::Error`]
impl Error for ErrorWithSuggestion {}
