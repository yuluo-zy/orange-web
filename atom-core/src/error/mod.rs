/// An allocation-optimized string.
pub type SharedString = std::borrow::Cow<'static, str>;

/// An owned dynamically typed error.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

use std::{error::Error as StdError, fmt};
use std::fmt::{Debug, Display, Formatter};
use http_body::Body;


#[derive(Debug)]
pub struct Error {
    message: SharedString,
    source: Option<Box<Error>>,
}

impl Error {
    /// Create a new `Error` from a supplied message.
    #[inline]
    pub fn new(error: impl Into<SharedString>) -> Self {
        Self {
            message: error.into(),
            source: None,
        }
    }
    #[inline]
    pub fn new_box(error: impl Into<BoxError>) -> Self {
        Self {
            message: SharedString::from(error.into().to_string()),
            source: None,
        }
    }

    pub fn with_source(error: impl Into<SharedString>, source: impl Into<Error>) -> Self {
        Self {
            message: error.into(),
            source: Some(Box::new(source.into())),
        }
    }

    #[inline]
    pub fn context(self, message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(self)),
        }
    }

    /// Returns the error message.
    #[inline]
    pub fn message(&self) -> &str {
        self.message.as_ref()
    }

    /// Returns the error source.
    #[inline]
    pub fn source(&self) -> Option<&Error> {
        self.source.as_deref()
    }

    /// Returns an iterator of the source errors contained by `self`.
    #[inline]
    pub fn sources(&self) -> Source<'_> {
        Source::new(self)
    }

    /// Returns the lowest level source of `self`.
    ///
    /// The root source is the last error in the iterator produced by [`sources()`](Error::sources).
    #[inline]
    pub fn root_source(&self) -> Option<&Error> {
        self.sources().last()
    }
}


impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        std::fmt::Debug::fmt(&self.message, f)
    }
}

impl StdError for Error {}

pub struct Source<'a> {
    next: Option<&'a Error>,
}

impl<'a> Source<'a> {
    #[inline]
    pub fn new(error: &'a Error) -> Self {
        Self {
            next: Some(error)
        }
    }
}

impl<'a> Iterator for Source<'a> {
    type Item = &'a Error;

    fn next(&mut self) -> Option<Self::Item> {
        let error = self.next?;
        self.next = error.source();
        Some(error)
    }
}
