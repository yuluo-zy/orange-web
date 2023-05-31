//! Defines storage for the remote address of the client

use crate::state::{FromState, State, StateData};
use std::net::SocketAddr;

struct ClientAddr {
    addr: SocketAddr,
}

impl StateData for ClientAddr {}

pub(crate) fn put_client_addr(state: &mut State, addr: SocketAddr) {
    state.put(ClientAddr { addr })
}

pub fn client_addr(state: &State) -> Option<SocketAddr> {
    // 获取提交IP todo:: 相同 IP 如何判断不同的请求内容
    ClientAddr::try_borrow_from(state).map(|c| c.addr)
}
