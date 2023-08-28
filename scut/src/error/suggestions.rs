use std::{error::Error, fmt};

/// Wraps anyhow::Error to add a related "Suggestion", which is a const str advising the user how to proceed
pub struct ErrorWithSuggestion {
    pub error: anyhow::Error,
    pub suggestion: &'static str,
}

pub trait ErrorSuggestions<T> {
    /// Add a suggestion to an [`Error`](std::error::Error) type, creating an [`anyhow::Error`] from it inside the [`ErrorWithSuggestion`]
    fn suggest(self, suggestion: &'static str) -> Result<T, ErrorWithSuggestion>;
}

impl<T, E> ErrorSuggestions<T> for Result<T, E>
where
    anyhow::Error: From<E>,
{
    fn suggest(self, suggestion: &'static str) -> Result<T, ErrorWithSuggestion> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(ErrorWithSuggestion {
                error: anyhow::Error::from(err),
                suggestion,
            }),
        }
    }
}

/// Our Display implementation forwards to the wrapped [`anyhow::Error`], ignoring the suggestion
impl fmt::Display for ErrorWithSuggestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}

/// The suggestion is appended for Debug format
impl fmt::Debug for ErrorWithSuggestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)?;
        write!(f, " > {}", self.suggestion)
    }
}

/// Implement [`Error`](std::error::Error) in order to support conversion into [`anyhow::Error`]
impl Error for ErrorWithSuggestion {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.error.source()
    }
}
