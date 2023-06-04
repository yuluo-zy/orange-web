//! Helpers for HTTP response generation

use crate::body::Body;
use hyper::header::{CONTENT_TYPE, LOCATION};
use hyper::{Method, Response, StatusCode};
use mime::Mime;
use std::borrow::Cow;

use crate::helpers::http::header::X_REQUEST_ID;
use crate::state::{request_id, FromState, State};

pub fn create_response<B>(state: &State, status: StatusCode, mime: Mime, body: B) -> Response<Body>
where
    B: Into<Body>,
{
    // use the basic empty response as a base
    let mut res = create_empty_response(state, status);

    // insert the content type header
    res.headers_mut()
        .insert(CONTENT_TYPE, mime.as_ref().parse().unwrap());

    // add the body on non-HEAD requests
    if Method::borrow_from(state) != Method::HEAD {
        *res.body_mut() = body.into();
    }

    res
}

pub fn create_empty_response(state: &State, status: StatusCode) -> Response<Body> {
    // new builder for the response
    let built = Response::builder()
        // always add status and req-id
        .status(status)
        .header(X_REQUEST_ID, request_id(state))
        // attach an empty body by default
        .body(Body::empty());

    // this expect should be safe due to generic bounds
    built.expect("Response built from a compatible type")
}

pub fn create_permanent_redirect<L: Into<Cow<'static, str>>>(
    state: &State,
    location: L,
) -> Response<Body> {
    let mut res = create_empty_response(state, StatusCode::PERMANENT_REDIRECT);
    res.headers_mut()
        .insert(LOCATION, location.into().to_string().parse().unwrap());
    res
}

pub fn create_temporary_redirect<L: Into<Cow<'static, str>>>(
    state: &State,
    location: L,
) -> Response<Body> {
    let mut res = create_empty_response(state, StatusCode::TEMPORARY_REDIRECT);
    res.headers_mut()
        .insert(LOCATION, location.into().to_string().parse().unwrap());
    res
}
