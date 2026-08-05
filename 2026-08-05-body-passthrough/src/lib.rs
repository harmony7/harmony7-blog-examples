mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{http_body, http_resp},
};

struct BodyPassthrough;

impl http_incoming::Guest for BodyPassthrough {
    fn handle(_request: http_incoming::Request, request_body: http_body::Body) -> Result<(), ()> {
        let response = http_resp::Response::new().map_err(|_| ())?;
        response
            .insert_header("content-type", b"text/plain")
            .map_err(|_| ())?;

        // The incoming request body, handed straight to send-downstream as the
        // outgoing response body. We never read a single byte of it ourselves.
        http_resp::send_downstream(response, request_body).map_err(|_| ())?;

        Ok(())
    }
}

bindings::export!(BodyPassthrough with_types_in bindings);
