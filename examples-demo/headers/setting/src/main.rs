//! Setting a header value for a Gotham web framework response
use atom_core::body::Body;
use atom_core::helpers::http::response::create_empty_response;
use atom_core::hyper::{ Response, StatusCode};
use atom_core::router::{build_simple_router, Router};
use atom_core::router::builder::{DefineSingleRoute, DrawRoutes};
use atom_core::state::State;

/// Create a `Handler` that adds a custom header.
pub fn handler(state: State) -> (State, Response<Body>) {
    let mut res = create_empty_response(&state, StatusCode::OK);

    {
        let headers = res.headers_mut();
        headers.insert("x-gotham", "Hello World!".parse().unwrap());
    };

    (state, res)
}

/// Create a `Router`
fn router() -> Router {
    build_simple_router(|route| {
        route.get("/").to(handler);
    })
}

/// Start a server and use a `Router` to dispatch requests
pub fn main() {
    simple_logger::init_with_env().expect("log 异常");
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    atom_core::start(addr, router()).unwrap();
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use gotham::test::TestServer;
//
//     #[test]
//     fn sets_header() {
//         let test_server = TestServer::new(|| Ok(handler)).unwrap();
//         let response = test_server
//             .client()
//             .get("http://localhost")
//             .perform()
//             .unwrap();
//
//         assert_eq!(response.status(), StatusCode::OK);
//         assert_eq!(response.headers().get("x-gotham").unwrap(), "Hello World!");
//     }
// }
