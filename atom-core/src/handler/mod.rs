use std::borrow::Cow;
use std::future::Future;
use std::ops::Deref;
use std::panic::RefUnwindSafe;
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use futures_util::future::{self, FutureExt};
use hyper::{Response, StatusCode};
use mime::{self, Mime};

use crate::helpers::http::response;
use crate::state::State;

mod assets;
pub use assets::*;

mod error;
use crate::body::Body;
pub use error::{HandlerError, MapHandlerError, MapHandlerErrorFuture};

pub type HandlerResult = Result<(State, Response<Body>), (State, HandlerError)>;

pub type SimpleHandlerResult = Result<Response<Body>, HandlerError>;

pub type HandlerFuture = dyn Future<Output = HandlerResult> + Send;

pub trait Handler: Send {
    fn handle(self, state: State) -> Pin<Box<HandlerFuture>>;
}

impl<F, R> Handler for F
where
    F: FnOnce(State) -> R + Send,
    R: IntoHandlerFuture,
{
    fn handle(self, state: State) -> Pin<Box<HandlerFuture>> {
        self(state).into_handler_future()
    }
}

pub trait NewHandler: Send + Sync + RefUnwindSafe {
    /// The type of `Handler` created by the `NewHandler`.
    type Instance: Handler + Send;

    /// Create and return a new `Handler` value.
    fn new_handler(&self) -> anyhow::Result<Self::Instance>;
}

impl<F, H> NewHandler for F
where
    F: Fn() -> anyhow::Result<H> + Send + Sync + RefUnwindSafe,
    H: Handler + Send,
{
    type Instance = H;

    fn new_handler(&self) -> anyhow::Result<H> {
        self()
    }
}

impl<H> NewHandler for Arc<H>
where
    H: NewHandler,
{
    type Instance = H::Instance;

    fn new_handler(&self) -> anyhow::Result<Self::Instance> {
        self.deref().new_handler()
    }
}

/// Represents a type which can be converted into the future type returned by a `Handler`.
///
/// This is used to allow functions with different return types to satisfy the `Handler` trait
/// bound via the generic function implementation.
pub trait IntoHandlerFuture {
    /// Converts this value into a boxed future resolving to a state and response.
    fn into_handler_future(self) -> Pin<Box<HandlerFuture>>;
}

impl<T> IntoHandlerFuture for (State, T)
where
    T: IntoResponse,
{
    fn into_handler_future(self) -> Pin<Box<HandlerFuture>> {
        let (state, t) = self;
        let response = t.into_response(&state);
        future::ok((state, response)).boxed()
    }
}

impl IntoHandlerFuture for Pin<Box<HandlerFuture>> {
    fn into_handler_future(self) -> Pin<Box<HandlerFuture>> {
        self
    }
}

pub trait IntoResponse {
    /// Converts this value into a `hyper::Response`
    fn into_response(self, state: &State) -> Response<Body>;
}

impl IntoResponse for Response<Body> {
    fn into_response(self, _state: &State) -> Response<Body> {
        self
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self, state: &State) -> Response<Body> {
        match self {
            Ok(res) => res.into_response(state),
            Err(e) => e.into_response(state),
        }
    }
}

impl<B> IntoResponse for (Mime, B)
where
    B: Into<Body>,
{
    fn into_response(self, state: &State) -> Response<Body> {
        (StatusCode::OK, self.0, self.1).into_response(state)
    }
}

impl<B> IntoResponse for (StatusCode, Mime, B)
where
    B: Into<Body>,
{
    fn into_response(self, state: &State) -> Response<Body> {
        response::create_response(state, self.0, self.1, self.2)
    }
}

// derive IntoResponse for Into<Body> types
macro_rules! derive_into_response {
    ($type:ty) => {
        impl IntoResponse for $type {
            fn into_response(self, state: &State) -> Response<Body> {
                (StatusCode::OK, mime::TEXT_PLAIN, self).into_response(state)
            }
        }
    };
}

// derive Into<Body> types - this is required because we
// can't impl IntoResponse for Into<Body> due to Response<T>
// and the potential it will add Into<Body> in the future
derive_into_response!(Bytes);
derive_into_response!(String);
derive_into_response!(Vec<u8>);
derive_into_response!(&'static str);
derive_into_response!(&'static [u8]);
derive_into_response!(Cow<'static, str>);
derive_into_response!(Cow<'static, [u8]>);
