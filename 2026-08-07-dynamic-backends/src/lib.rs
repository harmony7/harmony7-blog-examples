mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_req, http_resp},
};

/// Reads a single request header value, growing the buffer and retrying
/// if the ABI tells us the one we tried was too small.
fn get_header_value(
    request: &http_incoming::Request,
    name: &str,
) -> Result<Option<String>, http_req::Error> {
    let mut max_len: u64 = 128;
    loop {
        match request.get_header_value(name, max_len) {
            Ok(Some(bytes)) => return Ok(Some(String::from_utf8_lossy(&bytes).into_owned())),
            Ok(None) => return Ok(None),
            Err(http_req::Error::BufferLen(needed)) => max_len = needed,
            Err(e) => return Err(e),
        }
    }
}

struct DynamicBackends;

impl http_incoming::Guest for DynamicBackends {
    fn handle(request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        // WARNING: taking `target_host` straight from a request header with no
        // validation is an SSRF vulnerability, left unfixed here only to keep the
        // demo focused on `register_dynamic_backend`. Don't ship this as-is: an
        // attacker can point it at an internal admin endpoint, a cloud metadata
        // address, anywhere this service can reach that they can't. Validate
        // against a set of hosts you've decided are safe to reach first.
        let target_host = get_header_value(&request, "x-target-host")
            .map_err(|_| ())?
            .unwrap_or_else(|| "http-me.fastly.dev".to_string());

        let options = backend::DynamicBackendOptions::new();
        options.use_tls(true);

        let backend =
            backend::register_dynamic_backend("target", &target_host, options).map_err(|_| ())?;
        let is_dynamic = backend.is_dynamic().map_err(|_| ())?;

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
        let msg = format!(
            "Registered backend for {target_host} (is_dynamic: {is_dynamic})\n\
             Backend said:\n{backend_text}\n"
        );
        http_body::write(&body, msg.as_bytes()).map_err(|_| ())?;

        http_resp::send_downstream(response, body).map_err(|_| ())?;

        Ok(())
    }
}

bindings::export!(DynamicBackends with_types_in bindings);
