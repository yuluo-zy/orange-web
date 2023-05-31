//! Defines types for passing request state through `Middleware` and `Handler` implementations

pub(crate) mod client_addr;
mod data;
mod from_state;
mod request_id;

use hyper::http::request;
use hyper::upgrade::OnUpgrade;
use hyper::{ Request};
use log::{debug, trace};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::net::SocketAddr;
use crate::body::Body;

pub use crate::state::client_addr::client_addr;
pub use crate::state::data::StateData;
pub use crate::state::from_state::FromState;
pub use crate::state::request_id::request_id;

use crate::state::client_addr::put_client_addr;
pub(crate) use crate::state::request_id::set_request_id;

// https://docs.rs/http/0.2.5/src/http/extensions.rs.html#8-28
// With TypeIds as keys, there's no need to hash them. They are already hashes
// themselves, coming from the compiler. The IdHasher just holds the u64 of
// the TypeId, and then returns it, instead of doing any bit fiddling.
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }
}

// TODO:: 可灰度错误


pub struct State {
    data: HashMap<TypeId, Box<dyn Any + Send>, BuildHasherDefault<IdHasher>>,
}

// todo:: State 的生命周期问题

impl State {
    /// Creates a new, empty `State` container. This is for internal Gotham use, because the
    /// ability to create a new `State` container would allow for libraries and applications to
    /// incorrectly discard important internal data.
    pub(crate) fn new() -> State {
        State {
            data: HashMap::default(),
        }
    }

    /// Creates a new, empty `State` and yields it mutably into the provided closure. This is
    /// intended only for use in the documentation tests for `State`, since the `State` container
    /// cannot be constructed otherwise.
    #[doc(hidden)]
    pub fn with_new<F>(f: F)
    where
        F: FnOnce(&mut State),
    {
        f(&mut State::new())
    }

    /// Instantiate a new `State` for a given `Request`. This is primarily useful if you're calling
    /// Gotham from your own Hyper service.
    pub fn from_request(req: Request<Body>, client_addr: SocketAddr) -> Self {
        let mut state = Self::new();

        put_client_addr(&mut state, client_addr);

        let (
            request::Parts {
                method,
                uri,
                version,
                headers,
                mut extensions,
                ..
            },
            body,
        ) = req.into_parts();

        // 添加静态常量池

        // 将 请求体 解包然后 进行复制
        // state.put(RequestPathSegments::new(uri.path()));
        state.put(method);
        state.put(uri);
        state.put(version);
        state.put(headers);
        state.put(body);

        if let Some(on_upgrade) = extensions.remove::<OnUpgrade>() {
            state.put(on_upgrade);
        }

        {
            let request_id = set_request_id(&mut state);
            // 设置请求ID
            debug!(
                "[DEBUG][{}][Thread][{:?}]",
                request_id,
                std::thread::current().id(),
            );
        };

        state
    }


    pub fn put<T>(&mut self, t: T)
    where
        T: StateData,
    {
        let type_id = TypeId::of::<T>();
        trace!(" inserting record to state for type_id `{:?}`", type_id);
        // 上下文的数据 都是存放到堆内存中， 相同类型大 数据会出现数值覆盖情况
        self.data.insert(type_id, Box::new(t));
    }


    pub fn has<T>(&self) -> bool
    where
        T: StateData,
    {
        let type_id = TypeId::of::<T>();
        self.data.get(&type_id).is_some()
    }


    pub fn try_borrow<T>(&self) -> Option<&T>
    where
        T: StateData,
    {
        let type_id = TypeId::of::<T>();
        trace!(" borrowing state data for type_id `{:?}`", type_id);

        // 如果内部是 对应泛型实例， 则使用对应 的 引用返回数据， 返回借用， 是存在着
        self.data.get(&type_id).and_then(|b| b.downcast_ref::<T>())
    }


    pub fn borrow<T>(&self) -> &T
    where
        T: StateData,
    {
        // 直接返回对应的数据 而不是返回OPen的类型
        self.try_borrow()
            .expect("required type is not present in State container")
    }


    pub fn try_borrow_mut<T>(&mut self) -> Option<&mut T>
    where
        T: StateData,
    {
        let type_id = TypeId::of::<T>();
        trace!(" mutably borrowing state data for type_id `{:?}`", type_id);
        self.data
            .get_mut(&type_id)
            .and_then(|b| b.downcast_mut::<T>())
    }


    pub fn borrow_mut<T>(&mut self) -> &mut T
    where
        T: StateData,
    {
        self.try_borrow_mut()
            .expect("required type is not present in State container")
    }


    pub fn try_take<T>(&mut self) -> Option<T>
    where
        T: StateData,
    {
        let type_id = TypeId::of::<T>();
        trace!(
            " taking ownership from state data for type_id `{:?}`",
            type_id
        );
        self.data
            .remove(&type_id)
            .and_then(|b| b.downcast::<T>().ok())
            .map(|b| *b) // 转换成 option<T>
    }


    pub fn take<T>(&mut self) -> T
    where
        T: StateData,
    {
        self.try_take()
            .expect("required type is not present in State container")
    }
}
