use std::{error::Error, fmt};

use crate::error::ErrorWithSuggestion;

/// Custom report format for anyhow Errors.
///
/// Intended to be used as the return result of `main`
///
/// ## Examples
///
/// ```rust
/// # use scut_core::error::Report;
/// fn my_fallible_function() -> Result<(), Report> {
/// # const _: &str = stringify! {
///     ...
/// # }; Ok(())
/// }
///
/// fn main() -> Result<(), Report> {
///     my_fallible_function()
/// }
/// ```
pub struct Report(anyhow::Error);

impl fmt::Debug for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        report_error(&self.0, f)
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
/// Pretty print an anyhow::Error that may contain errors with suggestions via [`ErrorWithSuggestion`].
///
/// Intended for use by [`Report`].
fn report_error(error: &anyhow::Error, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut suggestions = Vec::new();

    write!(f, "Error: {}\n\n", error)?;

    let mut error_chain = error.chain().skip(1);

    if let Some(cause) = error_chain.next() {
        writeln!(f, "Caused by:\n\t* {cause}")?;
        report_cause(cause, f, &mut suggestions)?;
        for cause in error_chain {
            report_cause(cause, f, &mut suggestions)?;
        }
    }

    for suggestion in suggestions {
        report_suggestion(suggestion, f)?;
    }

    Ok(())
}

fn report_cause<'a>(
    cause: &'a (dyn Error + 'static),
    f: &mut fmt::Formatter<'_>,
    suggestions: &mut Vec<&'static str>,
) -> fmt::Result {
    if let Some(ErrorWithSuggestion {
        error: _,
        suggestion,
    }) = cause.downcast_ref::<ErrorWithSuggestion>()
    {
        suggestions.push(suggestion);
    }
    writeln!(f, "\t* {cause}")
}

fn report_suggestion(suggestion: &'static str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for suggestion in suggestion.lines() {
        writeln!(f, "> {suggestion}")?;
    }
    writeln!(f)
}

impl From<anyhow::Error> for Report {
    fn from(error: anyhow::Error) -> Self {
        Report(error)
    }
}
