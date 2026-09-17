// <fold imports, and a helper for sending a text response>
mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_resp},
};

fn respond(status: u16, text: String) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    response.set_status(status).map_err(|_| ())?;
    response.insert_header("content-type", b"text/plain").map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    http_resp::send_downstream(response, body).map_err(|_| ())
}
// </fold>

struct Fanout;

impl http_incoming::Guest for Fanout {
    fn handle(request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let origin = backend::Backend::open("origin").map_err(|_| ())?;

        // <highlight>
        match request.redirect_to_grip_proxy(&origin) {
            // Fanout holds the connection now, and asks the origin what to do with it.
            Ok(()) => Ok(()),
        // </highlight>
            Err(e) => respond(501, format!("redirect-to-grip-proxy: Err({e:?})\n")),
        }
    }
}

bindings::export!(Fanout with_types_in bindings);
