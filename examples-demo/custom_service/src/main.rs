//! An example usage of Gotham from another service.

use anyhow::{Context as _, Error};
use futures_util::future::{BoxFuture, FutureExt};
use atom_core::hyper::server::conn::http1;
use atom_core::hyper::service::Service;
use atom_core::hyper::{ Request, Response};
use atom_core::router::{build_simple_router, Router};
use atom_core::service::call_handler;
use atom_core::state::State;
use std::net::SocketAddr;
use std::panic::AssertUnwindSafe;
use std::task;
use tokio::net::TcpListener;
use atom_core::body::Body;
use atom_core::hyper::body::Incoming;
use atom_core::router::builder::{DefineSingleRoute, DrawRoutes};

#[derive(Clone)]
struct MyService {
    router: Router,
    addr: SocketAddr,
}

impl Service<Request<Incoming>> for MyService {
    type Response = Response<Body>;
    type Error = Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        // NOTE: You don't *have* to use call_handler for this (you could use `router.handle`), but
        // call_handler will catch panics and return en error response.
        let state = State::from_request_incoming(req, self.addr);
        call_handler(self.router.clone(), AssertUnwindSafe(state)).boxed()
    }
}

pub fn say_hello(state: State) -> (State, &'static str) {
    (state, "hello world")
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let router = build_simple_router(|route| {
        // For the path "/" invoke the handler "say_hello"
        route.get("/").to(say_hello);
    });

    let addr = "127.0.0.1:7878";
    let listener = TcpListener::bind(&addr).await?;

    println!("Listening for requests at http://{}", addr);

    loop {
        let (socket, addr) = listener
            .accept()
            .await
            .context("Error accepting connection")?;

        let service = MyService {
            router: router.clone(),
            addr,
        };

        let task = async move {
            http1::Builder::new()
                .serve_connection(socket, service)
                .await
                .context("Error serving connection")?;

            Result::<_, Error>::Ok(())
        };

        tokio::spawn(task);
    }
}
