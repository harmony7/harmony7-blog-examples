mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_req, http_resp},
};

use http_resp::PendingResponseKind::{Any, Error, Response};

/// Starts a request, queues header changes for whatever comes back, and
/// hands the pending response to Compute to forward. Returns without
/// waiting for the backend.
fn forward(backend: &backend::Backend, uri: &str) -> Result<(), ()> {
    let request = http_req::Request::new().map_err(|_| ())?;
    request.set_uri(uri).map_err(|_| ())?;
    let pending = http_req::send_async(request, http_body::new().map_err(|_| ())?, backend)
        .map_err(|_| ())?;

    // On every response, whichever kind it turns out to be.
    http_resp::insert_header_pending(&pending, "x-via", b"compute", Any).map_err(|_| ())?;
    // Only on a response that came from the backend.
    http_resp::append_header_pending(&pending, "x-outcome", b"origin", Response)
        .map_err(|_| ())?;
    http_resp::remove_header_pending(&pending, "x-internal", Response).map_err(|_| ())?;
    // Only on the 5XX that Compute makes up if the request fails.
    http_resp::append_header_pending(&pending, "x-outcome", b"synthetic", Error)
        .map_err(|_| ())?;
    http_resp::append_header_pending(&pending, "retry-after", b"5", Error).map_err(|_| ())?;

    http_resp::send_downstream_pending(pending).map_err(|_| ())
}

struct PendingResponseHeaders;

impl http_incoming::Guest for PendingResponseHeaders {
    fn handle(request: http_incoming::Request, _body: http_body::Body) -> Result<(), ()> {
        let path = request.get_uri(1024).map_err(|_| ())?;
        let path = path.rsplit_once('/').map(|(_, p)| p).unwrap_or("");

        let http_me = backend::Backend::open("http_me").map_err(|_| ())?;
        match path {
            // http-me adds an x-internal header, which the program removes.
            "ok" => forward(&http_me, "https://http-me.fastly.dev/header=x-internal:secret"),
            // http-me waits two seconds before answering.
            "slow" => forward(&http_me, "https://http-me.fastly.dev/wait=2000"),
            // A name that can never resolve, so the request fails.
            "fail" => {
                let broken = backend::register_dynamic_backend(
                    "broken",
                    "no-such-host.invalid",
                    backend::DynamicBackendOptions::new(),
                )
                .map_err(|_| ())?;
                forward(&broken, "http://no-such-host.invalid/")
            }
            _ => {
                let response = http_resp::Response::new().map_err(|_| ())?;
                response.set_status(404).map_err(|_| ())?;
                let body = http_body::new().map_err(|_| ())?;
                http_resp::send_downstream(response, body).map_err(|_| ())
            }
        }
    }
}

bindings::export!(PendingResponseHeaders with_types_in bindings);
