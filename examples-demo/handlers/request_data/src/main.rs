//! A basic example showing the request components
use futures_util::future::{self, FutureExt};
use std::pin::Pin;
use atom_core::body::Body;

use atom_core::handler::HandlerFuture;
use atom_core::helpers::http::response::create_empty_response;
use atom_core::hyper::{HeaderMap, Method, Response, StatusCode, Uri, Version};
use atom_core::hyper::http::HeaderValue;
use atom_core::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};
use atom_core::router::Router;
use atom_core::state::{FromState, State};

/// Extract the main elements of the request except for the `Body`
fn print_request_elements(state: &State) {
    let method = Method::borrow_from(state);
    let uri = Uri::borrow_from(state);
    let http_version = Version::borrow_from(state);
    // let headers = state.borrow::<HeaderMap>();
    let headers = HeaderMap::<HeaderValue>::borrow_from(state);
    println!("Method: {:?}", method);
    println!("URI: {:?}", uri);
    println!("HTTP Version: {:?}", http_version);
    println!("Headers: {:?}", headers);
}

/// Extracts the elements of the POST request and prints them
fn post_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    print_request_elements(&state);
    let f = Body::take_from(&mut state).to_bytes().then(|full_body| match full_body {
        Ok(valid_body) => {
            let body_content = String::from_utf8(valid_body.to_vec()).unwrap();
            println!("Body: {}", body_content);
            let res = create_empty_response(&state, StatusCode::OK);
            future::ok((state, res))
        }
        Err(e) => future::err((state, e.into())),
    });

    f.boxed()
}

/// Show the GET request components by printing them.
fn get_handler(state: State) -> (State, Response<Body>) {
    print_request_elements(&state);
    let res = create_empty_response(&state, StatusCode::OK);
    (state, res)
}

/// Create a `Router`
fn router() -> Router {
    build_simple_router(|route| {
        route.associate("/", |assoc| {
            assoc.get().to(get_handler);
            assoc.post().to(post_handler);
        });
    })
}

/// Start a server and use a `Router` to dispatch requests
pub fn main() {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    atom_core::start(addr, router()).unwrap();
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use gotham::mime::TEXT_PLAIN;
//     use gotham::test::TestServer;
//
//     #[test]
//     fn get_request() {
//         let test_server = TestServer::new(router()).unwrap();
//         let response = test_server
//             .client()
//             .get("http://localhost")
//             .perform()
//             .unwrap();
//
//         assert_eq!(response.status(), StatusCode::OK);
//     }
//
//     #[test]
//     fn post_request() {
//         let test_server = TestServer::new(router()).unwrap();
//         let response = test_server
//             .client()
//             .post("http://localhost", "", TEXT_PLAIN)
//             .perform()
//             .unwrap();
//
//         assert_eq!(response.status(), StatusCode::OK);
//     }
// }
