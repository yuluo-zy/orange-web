use hyper::{Response};
use serde::{Deserialize, Deserializer};
use crate::body::Body;
use crate::router::response::StaticResponseExtender;

// use crate::router::response::StaticResponseExtender;
use crate::state::{State, StateData};

pub trait QueryStringExtractor<B>:
    for<'de> Deserialize<'de> + StaticResponseExtender<ResBody = B> + StateData
where
    B: HttpBody,
{
}

impl<T, B> QueryStringExtractor<B> for T
where
    B: HttpBody,
    for<'de> T: Deserialize<'de> + StaticResponseExtender<ResBody = B> + StateData,
{
}

/// A `QueryStringExtractor` that does not extract/store any data.
///
/// This is the default `QueryStringExtractor` which is applied to a route when no other
/// `QueryStringExtractor` is provided. It ignores any query parameters, and always succeeds during
/// deserialization.
#[derive(Debug)]
pub struct NoopQueryStringExtractor;

// This doesn't get derived correctly if we just `#[derive(Deserialize)]` above, because the
// Deserializer expects to _ignore_ a value, not just do nothing. By filling in the impl ourselves,
// we can explicitly do nothing.
impl<'de> Deserialize<'de> for NoopQueryStringExtractor {
    fn deserialize<D>(_de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(NoopQueryStringExtractor)
    }
}

impl StateData for NoopQueryStringExtractor {}

impl StaticResponseExtender for NoopQueryStringExtractor {
    type ResBody = Body;
    fn extend(_state: &mut State, _res: &mut Response<Body>) {}
}
