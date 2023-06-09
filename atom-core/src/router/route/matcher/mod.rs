//! Defines the type `RouteMatcher` and default implementations.

mod accept;
mod access_control_request_method;
mod and;
mod any;
mod content_type;

pub use self::accept::AcceptHeaderRouteMatcher;
pub use self::access_control_request_method::AccessControlRequestMethodMatcher;
pub use self::and::AndRouteMatcher;
pub use self::any::AnyRouteMatcher;
pub use self::content_type::ContentTypeHeaderRouteMatcher;

mod lookup_table;
use self::lookup_table::{LookupTable, LookupTableFromTypes};

use std::panic::RefUnwindSafe;

use hyper::{Method, StatusCode};
use log::trace;

use crate::router::non_match::RouteNonMatch;
use crate::state::{request_id, FromState, State};

/// Determines if conditions required for the associated `Route` to be invoked by the `Router` have
/// been met.
pub trait RouteMatcher: RefUnwindSafe + Clone {
    /// Determines if the `Request` meets pre-defined conditions.
    fn is_match(&self, state: &State) -> Result<(), RouteNonMatch>;
}

/// Allow various types to represent themselves as a `RouteMatcher`
pub trait IntoRouteMatcher {
    /// The concrete RouteMatcher each implementation will provide.
    type Output: RouteMatcher;

    /// Transform into a `RouteMatcher` of the the associated type identified by `Output`.
    fn into_route_matcher(self) -> Self::Output;
}

impl IntoRouteMatcher for Vec<Method> {
    type Output = MethodOnlyRouteMatcher;

    fn into_route_matcher(self) -> Self::Output {
        MethodOnlyRouteMatcher::new(self)
    }
}

impl<M> IntoRouteMatcher for M
where
    M: RouteMatcher + Send + Sync + 'static,
{
    type Output = M;

    fn into_route_matcher(self) -> Self::Output {
        self
    }
}

/// A `RouteMatcher` that succeeds when the `Request` has been made with an accepted HTTP request
/// method.
///
/// # Examples
///
/// ```rust
/// # extern crate gotham;
/// # extern crate hyper;
/// # fn main() {
/// #   use hyper::Method;
/// #   use gotham::state::State;
/// #   use gotham::router::route::matcher::{RouteMatcher, MethodOnlyRouteMatcher};
/// #
/// #   State::with_new(|state| {
/// #
/// let methods = vec![Method::GET, Method::HEAD];
/// let matcher = MethodOnlyRouteMatcher::new(methods);
///
/// state.put(Method::GET);
/// assert!(matcher.is_match(&state).is_ok());
///
/// state.put(Method::POST);
/// assert!(matcher.is_match(&state).is_err());
/// #   });
/// # }
/// ```
#[derive(Clone)]
pub struct MethodOnlyRouteMatcher {
    methods: Vec<Method>,
}

impl MethodOnlyRouteMatcher {
    /// Creates a new `MethodOnlyRouteMatcher`.
    pub fn new(methods: Vec<Method>) -> Self {
        MethodOnlyRouteMatcher { methods }
    }
}

impl RouteMatcher for MethodOnlyRouteMatcher {
    /// Determines if the `Request` was made using a `Method` the instance contains.
    fn is_match(&self, state: &State) -> Result<(), RouteNonMatch> {
        let method = Method::borrow_from(state);
        if self.methods.iter().any(|m| m == method) {
            trace!(
                "[{}] matched request method {} to permitted method",
                request_id(state),
                method
            );
            Ok(())
        } else {
            trace!(
                "[{}] did not match request method {}",
                request_id(state),
                method
            );
            Err(RouteNonMatch::new(StatusCode::METHOD_NOT_ALLOWED)
                .with_allow_list(self.methods.as_slice()))
        }
    }
}
