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

    writeln!(f, "{error}")?;

    let mut error_chain = error.chain().skip(1).peekable();
    if error_chain.peek().is_some() {
        writeln!(f, "\nCaused by:")?;
    }
    for cause in error_chain {
        report_cause(cause, f, &mut suggestions)?;
    }

    if !suggestions.is_empty() {
        writeln!(f, "\n")?;
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
    writeln!(f, "  - {cause}")
}

fn report_suggestion(suggestion: &'static str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for suggestion in suggestion.lines() {
        writeln!(f, "> {suggestion}")?;
    }
    Ok(())
}

impl From<anyhow::Error> for Report {
    fn from(error: anyhow::Error) -> Self {
        Report(error)
    }
}
