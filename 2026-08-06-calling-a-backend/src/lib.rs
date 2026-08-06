mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_req, http_resp},
};

struct CallingABackend;

impl http_incoming::Guest for CallingABackend {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let backend = backend::Backend::open("http_me").map_err(|_| ())?;

        let out_request = http_req::Request::new().map_err(|_| ())?;
        out_request.set_method("GET").map_err(|_| ())?;
        out_request.set_uri("/anything").map_err(|_| ())?;

        let out_body = http_body::new().map_err(|_| ())?;

        let (_backend_response, backend_body) =
            http_req::send(out_request, out_body, &backend).map_err(|_| ())?;

        let mut buf = Vec::new();
        loop {
            let chunk = http_body::read(&backend_body, 8192).map_err(|_| ())?;
            if chunk.is_empty() {
                break;
            }
            buf.extend_from_slice(&chunk);
        }
        let backend_text = String::from_utf8_lossy(&buf).into_owned();

        let response = http_resp::Response::new().map_err(|_| ())?;
        response
            .insert_header("content-type", b"text/plain")
            .map_err(|_| ())?;

        let body = http_body::new().map_err(|_| ())?;
        let msg = format!("Backend said:\n{backend_text}\n");
        http_body::write(&body, msg.as_bytes()).map_err(|_| ())?;

        http_resp::send_downstream(response, body).map_err(|_| ())?;

        Ok(())
    }
}

bindings::export!(CallingABackend with_types_in bindings);
