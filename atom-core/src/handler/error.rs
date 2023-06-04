use futures_util::future::FusedFuture;
use std::fmt::{Debug, Display};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::body::Body;
use hyper::{Response, StatusCode};
use log::{debug, trace};

use crate::handler::IntoResponse;
use crate::helpers::http::response::create_empty_response;
use crate::state::{request_id, State};

/// Describes an error which occurred during handler execution, and allows the creation of a HTTP
/// `Response`.
#[derive(Debug)]
pub struct HandlerError {
    status_code: StatusCode,
    cause: anyhow::Error,
}

/// Convert a generic `anyhow::Error` into a `HandlerError`, similar as you would a concrete error
/// type with `into_handler_error()`.
impl<E> From<E> for HandlerError
where
    E: Into<anyhow::Error> + Display,
{
    fn from(error: E) -> HandlerError {
        trace!(" converting Error to HandlerError: {}", error);

        HandlerError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            cause: error.into(),
        }
    }
}

impl HandlerError {
    /// Returns the HTTP status code associated with this `HandlerError`.
    pub fn status(&self) -> StatusCode {
        self.status_code
    }

    pub fn with_status(self, status_code: StatusCode) -> HandlerError {
        HandlerError {
            status_code,
            ..self
        }
    }

    /// Returns the cause of this error by reference.
    pub fn cause(&self) -> &anyhow::Error {
        &self.cause
    }

    /// Returns the cause of this error.
    pub fn into_cause(self) -> anyhow::Error {
        self.cause
    }

    /// Attempt to downcast the cause by reference.
    pub fn downcast_cause_ref<E>(&self) -> Option<&E>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        self.cause.downcast_ref()
    }

    /// Attempt to downcast the cause by mutable reference.
    pub fn downcast_cause_mut<E>(&mut self) -> Option<&mut E>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        self.cause.downcast_mut()
    }
}

impl IntoResponse for HandlerError {
    fn into_response(self, state: &State) -> Response<Body> {
        debug!(
            "[{}] HandlerError generating {} {} response: {}",
            request_id(state),
            self.status_code.as_u16(),
            self.status_code
                .canonical_reason()
                .unwrap_or("(unregistered)",),
            self.cause
        );

        create_empty_response(state, self.status_code)
    }
}

pub trait MapHandlerError<T> {
    /// Equivalent of `map_err(|err| HandlerError::from(err).with_status(status_code))`.
    fn map_err_with_status(self, status_code: StatusCode) -> Result<T, HandlerError>;
}

impl<T, E> MapHandlerError<T> for Result<T, E>
where
    E: Into<anyhow::Error> + Display,
{
    fn map_err_with_status(self, status_code: StatusCode) -> Result<T, HandlerError> {
        self.map_err(|err| {
            trace!(" converting Error to HandlerError: {}", err);
            HandlerError {
                status_code,
                cause: err.into(),
            }
        })
    }
}

// The future for `map_err_with_status`.
#[pin_project::pin_project(project = MapErrWithStatusProj, project_replace = MapErrWithStatusProjOwn)]
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub enum MapErrWithStatus<F> {
    Incomplete {
        #[pin]
        future: F,
        status: StatusCode,
    },
    Complete,
}

impl<F> MapErrWithStatus<F> {
    fn new(future: F, status: StatusCode) -> Self {
        Self::Incomplete { future, status }
    }
}

impl<F, T, E> FusedFuture for MapErrWithStatus<F>
where
    F: Future<Output = Result<T, E>>,
    E: Into<anyhow::Error> + Display,
{
    fn is_terminated(&self) -> bool {
        matches!(self, Self::Complete)
    }
}

impl<F, T, E> Future for MapErrWithStatus<F>
where
    F: Future<Output = Result<T, E>>,
    E: Into<anyhow::Error> + Display,
{
    type Output = Result<T, HandlerError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.as_mut().project() {
            MapErrWithStatusProj::Incomplete { future, .. } => {
                let output = match future.poll(cx) {
                    Poll::Ready(output) => output,
                    Poll::Pending => return Poll::Pending,
                };
                match self.project_replace(MapErrWithStatus::Complete) {
                    MapErrWithStatusProjOwn::Incomplete { status, .. } => {
                        Poll::Ready(output.map_err_with_status(status))
                    }
                    MapErrWithStatusProjOwn::Complete => unreachable!(),
                }
            }
            MapErrWithStatusProj::Complete => {
                panic!("MapErrWithStatus must not be polled after it returned `Poll::Ready`")
            }
        }
    }
}

pub trait MapHandlerErrorFuture {
    /// Equivalent of `map_err(|err| HandlerError::from(err).with_status(status_code))`.
    fn map_err_with_status(self, status_code: StatusCode) -> MapErrWithStatus<Self>
    where
        Self: Sized;
}

impl<T, E, F> MapHandlerErrorFuture for F
where
    E: Into<anyhow::Error> + Display,
    F: Future<Output = Result<T, E>>,
{
    fn map_err_with_status(self, status_code: StatusCode) -> MapErrWithStatus<Self> {
        MapErrWithStatus::new(self, status_code)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io;
    use thiserror::Error;

    #[derive(Debug, Error)]
    #[error("Dummy Error")]
    struct DummyError;

    fn error_prone() -> Result<(), HandlerError> {
        Err(DummyError.into())
    }

    #[test]
    fn test_error_downcast() {
        let mut err = error_prone().unwrap_err();
        assert!(err.downcast_cause_ref::<DummyError>().is_some());
        assert!(err.downcast_cause_mut::<DummyError>().is_some());
        assert!(err.downcast_cause_ref::<io::Error>().is_none());
        assert!(err.downcast_cause_mut::<io::Error>().is_none());
    }
}