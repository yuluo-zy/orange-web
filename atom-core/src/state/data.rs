use std::any::Any;
use cookie::CookieJar;

use hyper::upgrade::OnUpgrade;
use hyper::{HeaderMap, Method, Uri, Version};
use crate::body::Body;

use crate::state::request_id::RequestId;




pub trait StateData: Any + Send {}

// TODO: Body 在 hyper 0.14 中 是 没有限定 !sync 的, 当前的实现是 允许他进行跨线程访问

impl StateData for Body {}
impl StateData for Method {}
impl StateData for Uri {}
impl StateData for Version {}
impl StateData for HeaderMap {}
impl StateData for OnUpgrade {}

impl StateData for RequestId {}
