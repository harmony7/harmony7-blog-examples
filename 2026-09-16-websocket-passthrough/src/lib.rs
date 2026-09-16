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

struct WebsocketPassthrough;

impl http_incoming::Guest for WebsocketPassthrough {
    fn handle(request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let echo = backend::Backend::open("echo").map_err(|_| ())?;

        let upgrade = request.get_header_value("upgrade", 64).ok().flatten();
        let is_websocket = upgrade.is_some_and(|v| v.eq_ignore_ascii_case(b"websocket"));

        if is_websocket {
            // <highlight>
            match request.redirect_to_websocket_proxy(&echo) {
                // Fastly holds the connection from here. Return without responding.
                Ok(()) => return Ok(()),
            // </highlight>
                Err(e) => {
                    return respond(501, format!("redirect-to-websocket-proxy: Err({e:?})\n"));
                }
            }
        }

        respond(200, "Not a WebSocket request. Send one with Upgrade: websocket.\n".to_string())
    }
}

bindings::export!(WebsocketPassthrough with_types_in bindings);
