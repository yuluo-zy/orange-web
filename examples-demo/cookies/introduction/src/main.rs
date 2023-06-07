//! An introduction to storing and retrieving cookie data, with the Gotham
//! web framework.
use atom_core::body::Body;
use atom_core::cookie::{Cookie, CookieJar};
use atom_core::helpers::http::response::create_response;
use atom_core::hyper::header::SET_COOKIE;
use atom_core::hyper::{Response, StatusCode};
use atom_core::middleware::cookie::CookieParser;
use atom_core::mime::TEXT_PLAIN;
use atom_core::pipeline::{new_pipeline, single_pipeline};
use atom_core::router::{build_router, Router};
use atom_core::router::builder::{DefineSingleRoute, DrawRoutes};
use atom_core::state::{FromState, State};

/// The first request will set a cookie, and subsequent requests will echo it back.
fn handler(state: State) -> (State, Response<Body>) {
    // Define a narrow scope so that state can be borrowed/moved later in the function.
    let adjective = {
        // Retrieve the cookie from the jar stored on the state.
        CookieJar::borrow_from(&state)
            .get("adjective")
            .map(|adj_cookie| adj_cookie.value().to_owned())
            .unwrap_or_else(|| "first time".to_string())
    };

    let mut response = create_response(
        &state,
        StatusCode::OK,
        TEXT_PLAIN,
        format!("Hello {} visitor\n", adjective),
    );

    {
        let cookie = Cookie::build("adjective", "repeat")
            .http_only(true)
            .finish();
        response
            .headers_mut()
            .append(SET_COOKIE, cookie.to_string().parse().unwrap());
    }

    (state, response)
}

/// Create a `Router`
fn router() -> Router {
    let (chain, pipelines) = single_pipeline(new_pipeline().add(CookieParser).build());
    build_router(chain, pipelines, |route| {
        route.get("/").to(handler);
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
//     use cookie::Cookie;
//     use gotham::hyper::header::COOKIE;
//     use gotham::test::TestServer;
//
//     #[test]
//     fn cookie_is_set_and_counter_increments() {
//         let test_server = TestServer::new(router()).unwrap();
//         let response = test_server
//             .client()
//             .get("http://localhost/")
//             .perform()
//             .unwrap();
//
//         assert_eq!(response.status(), StatusCode::OK);
//
//         assert_eq!(response.headers().get_all(SET_COOKIE).iter().count(), 1);
//
//         assert_eq!(
//             response
//                 .headers()
//                 .get(SET_COOKIE)
//                 .map(|hv| hv.to_str().unwrap()),
//             Some("adjective=repeat; HttpOnly")
//         );
//
//         let body = response.read_body().unwrap();
//         assert_eq!(&body[..], "Hello first time visitor\n".as_bytes());
//
//         let cookie = Cookie::new("adjective", "repeat");
//
//         let response = test_server
//             .client()
//             .get("http://localhost/")
//             .with_header(COOKIE, cookie.to_string().parse().unwrap())
//             .perform()
//             .unwrap();
//
//         assert_eq!(response.status(), StatusCode::OK);
//         let body = response.read_body().unwrap();
//         assert_eq!(&body[..], "Hello repeat visitor\n".as_bytes());
//     }
// }
