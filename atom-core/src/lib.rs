pub mod body;

pub mod extractor;
pub mod handler;
pub mod helpers;
pub mod middleware;
pub mod pipeline;
pub mod plain;
pub mod router;
pub mod service;
pub mod state;
pub mod tls;
pub mod test;
pub mod error;

pub use anyhow;
/// Re-export hyper
pub use hyper;
/// Re-export mime
pub use mime;
pub use cookie;
pub use reqwest  as client;
pub use atom_derive;

pub use plain::*;

use crate::handler::NewHandler;
use crate::service::GothamService;
use hyper::server::conn::http1;
use std::future::Future;
use std::io;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime;
use tokio::runtime::Runtime;

/// The error that can occur when starting the gotham server.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StartError {
    /// I/O error.
    #[error("I/O Error: {0}")]
    IoError(#[from] io::Error),
}

fn new_runtime(threads: usize) -> Runtime {
    runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .thread_name("gotham-worker")
        .enable_all()
        .build()
        .unwrap()
}

async fn tcp_listener<A>(addr: A) -> io::Result<TcpListener>
where
    A: ToSocketAddrs + 'static,
{
    let addr = addr.to_socket_addrs()?.next().ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "unable to resolve listener address")
    })?;
    TcpListener::bind(addr).await
}

/// Returns a `Future` used to spawn a Gotham application.
///
/// This is used internally, but it's exposed for clients that want to set up their own TLS
/// support. The wrap argument is a function that will receive a tokio-io TcpStream and should wrap
/// the socket as necessary. Errors returned by this function will be ignored and the connection
/// will be dropped if the future returned by the wrapper resolves to an error.
pub async fn bind_server<'a, NH, F, Wrapped, Wrap>(
    listener: TcpListener,
    new_handler: NH,
    wrap: Wrap,
) -> !
where
    NH: NewHandler + 'static,
    F: Future<Output = Result<Wrapped, ()>> + Unpin + Send + 'static,
    Wrapped: Unpin + AsyncRead + AsyncWrite + Send + 'static,
    Wrap: Fn(TcpStream) -> F,
{
    let protocol = Arc::new(http1::Builder::new());
    let gotham_service = GothamService::new(new_handler);

    loop {
        let (socket, addr) = match listener.accept().await {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("Socket Error: {}", err);
                continue;
            }
        };

        let service = gotham_service.connect(addr);
        let accepted_protocol = protocol.clone();
        let wrapper = wrap(socket);

        // NOTE: HTTP protocol errors and handshake errors are ignored here (i.e. so the socket
        // will be dropped).
        let task = async move {
            let socket = wrapper.await?;

            accepted_protocol
                .serve_connection(socket, service)
                .with_upgrades()
                .await
                .expect("accepted protocol errors");

            Result::<_, ()>::Ok(())
        };

        tokio::spawn(task);
    }
}
