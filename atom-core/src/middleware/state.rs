//! State driven middleware to enable attachment of values to request state.
//!
//! This module provides generics to enable attaching (appropriate) values to
//! the state of a request, through the use of `Middleware`. Middleware can
//! be created via `StateMiddleware::with`, with the provided value being the
//! value to attach to the request state.
use std::any::Any;
use crate::handler::HandlerFuture;
use crate::middleware::{Middleware, MiddlewareBuild};
use crate::state::{State};
use std::panic::RefUnwindSafe;
use std::pin::Pin;

/// Middleware binding for generic types to enable easy shared state.
///
/// This acts as nothing more than a `Middleware` instance which will
/// attach a generic type to a request `State`, however it removes a
/// barrier for users to Gotham who might not know the internals.
///
/// The generic types inside this struct can (and will) be cloned
/// often, so wrap your expensive types in reference counts as needed.
#[derive(Clone)]
pub struct StateMiddleware<T>
where
    T: Clone + RefUnwindSafe + Any + Send + Sync,
{
    t: T,
}

/// Main implementation.
impl<T> StateMiddleware<T>
where
    T: Clone + RefUnwindSafe + Any + Send + Sync,
{
    /// Creates a new middleware binding, taking ownership of the state data.
    pub fn new(t: T) -> Self {
        Self { t }
    }
}

/// `Middleware` trait implementation.
impl<T> Middleware for StateMiddleware<T>
where
    T: Clone + RefUnwindSafe + Any + Send + Sync,
{
    /// Attaches the inner generic value to the request state.
    ///
    /// This will enable the `Handler` to borrow the value directly from the state.
    fn call<Chain>(self, mut state: State, chain: Chain) -> Pin<Box<HandlerFuture>>
    where
        Chain: FnOnce(State) -> Pin<Box<HandlerFuture>>,
    {
        state.put(self.t);
        chain(state)
    }
}

/// `NewMiddleware` trait implementation.
impl<T> MiddlewareBuild for StateMiddleware<T>
where
    T: Clone + RefUnwindSafe + Any + Send + Sync,
{
    type Instance = Self;

    /// Clones the current middleware to a new instance.
    fn new_middleware(&self) -> anyhow::Result<Self::Instance> {
        Ok(self.clone())
    }
}
