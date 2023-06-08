use std::any::Any;
use hyper::{Response};
use hyper::body::Body as HttpBody;
use serde::{Deserialize, Deserializer};
use crate::body::Body;

use crate::router::response::StaticResponseExtender;
use crate::state::{State};

pub trait PathExtractor<B>:
// 指定一个更高级的生命周期
    for<'de> Deserialize<'de> + StaticResponseExtender<ResBody=B> + Any + Send
    where
        B: HttpBody,
{}

impl<T, B> PathExtractor<B> for T
    where
        B: HttpBody,
        for<'de> T: Deserialize<'de> + StaticResponseExtender<ResBody=B> + Any + Send,
{}

/// A `PathExtractor` that does not extract/store any data from the `Request` path.
///
/// This is the default `PathExtractor` which is applied to a route when no other `PathExtractor`
/// is provided. It ignores any dynamic path segments, and always succeeds during deserialization.
pub struct NoopPathExtractor;

// This doesn't get derived correctly if we just `#[derive(Deserialize)]` above, because the
// Deserializer expects to _ignore_ a value, not just do nothing. By filling in the impl ourselves,
// we can explicitly do nothing.
impl<'de> Deserialize<'de> for NoopPathExtractor {
    fn deserialize<D>(_de: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        Ok(NoopPathExtractor)
    }
}

impl StaticResponseExtender for NoopPathExtractor {
    type ResBody = Body;
    fn extend(_state: &mut State, _res: &mut Response<Body>) {}
}
