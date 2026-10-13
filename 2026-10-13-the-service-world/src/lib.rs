mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{http_body, http_resp},
};

fn respond(text: &str) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    http_resp::send_downstream(response, body).map_err(|_| ())
}

struct TheServiceWorld;

impl http_incoming::Guest for TheServiceWorld {
    fn handle(_request: http_incoming::Request, _body: http_body::Body) -> Result<(), ()> {
        respond("handled\n")
    }
}

bindings::export!(TheServiceWorld with_types_in bindings);
