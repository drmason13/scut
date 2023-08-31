use std::{error::Error, fmt};

/// Use this to check that certain errors are raised.
/// * mocks will generally create these and wrap them in anyhow
/// * tests will make the mock, run it in a way that ought to error and then attempt to downcast the anyhow::Error into this
#[derive(Debug, Clone, PartialEq)]
pub struct MockError<T>(T);

impl<T: fmt::Debug> MockError<T> {
    pub fn new(thing: T) -> Self {
        MockError(thing)
    }
}

impl<T: fmt::Debug> Error for MockError<T> {}

impl<T: fmt::Debug> fmt::Display for MockError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

/// Use this to raise an error if something wrong/unexpected happens in an error handling test to indicate that the test has failed!
#[derive(Debug, Clone, PartialEq)]
pub struct TestError;

impl Error for TestError {}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "<<<TEST ERROR>>>".fmt(f)
    }
}
