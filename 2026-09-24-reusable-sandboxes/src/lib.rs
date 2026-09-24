// <fold imports, and helpers for reading the request ID and sending a text response>
mod bindings;

use std::sync::atomic::{AtomicU32, Ordering};

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{compute_runtime, http_body, http_downstream, http_req, http_resp},
};

fn request_id(request: &http_req::Request) -> String {
    match http_downstream::downstream_client_request_id(request, 128) {
        Ok(id) => id,
        Err(e) => format!("Err({e:?})"),
    }
}

fn respond(text: String) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    response.insert_header("content-type", b"text/plain").map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    http_resp::send_downstream(response, body).map_err(|_| ())
}
// </fold>

// Survives from one request to the next, for as long as the sandbox does.
// <highlight>
static HANDLED: AtomicU32 = AtomicU32::new(0);
// </highlight>

fn serve_one(request: &http_req::Request) -> Result<(), ()> {
    let count = HANDLED.fetch_add(1, Ordering::SeqCst) + 1;
    respond(format!(
        "sandbox {}\nrequest {}\nrequests this sandbox has handled: {count}\n",
        compute_runtime::get_sandbox_id(),
        request_id(request),
    ))
}

struct ReusableSandboxes;

impl http_incoming::Guest for ReusableSandboxes {
    fn handle(request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let mut request = request;

        loop {
            serve_one(&request)?;

            // Offer to take another request, and wait up to five seconds for one.
            let options = http_downstream::NextRequestOptions { timeout_ms: Some(5_000), extra: None };
            // <highlight>
            let pending = match http_downstream::next_request(&options) {
            // </highlight>
                Ok(pending) => pending,
                Err(e) => {
                    println!("next-request: Err({e:?})");
                    return Ok(());
                }
            };

            // <highlight>
            match http_downstream::await_request(pending) {
                Ok(Some((next, _body))) => request = next,
            // </highlight>
                Ok(None) => {
                    println!("await-request: no more requests for this sandbox");
                    return Ok(());
                }
                Err(e) => {
                    println!("await-request: Err({e:?})");
                    return Ok(());
                }
            }
        }
    }
}

bindings::export!(ReusableSandboxes with_types_in bindings);
