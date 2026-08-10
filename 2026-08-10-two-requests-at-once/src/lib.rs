mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{async_io, backend, http_body, http_req, http_resp},
};
use std::time::Instant;

struct TwoRequestsAtOnce;

impl http_incoming::Guest for TwoRequestsAtOnce {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let backend = backend::Backend::open("http_me").map_err(|_| ())?;

        let mut pending: Vec<(&str, http_req::PendingResponse)> = Vec::new();
        for (label, wait_ms) in [("slow", 3000u32), ("fast", 1000u32)] {
            let out_request = http_req::Request::new().map_err(|_| ())?;
            out_request.set_method("GET").map_err(|_| ())?;
            out_request
                .set_uri(&format!("/anything?wait={wait_ms}"))
                .map_err(|_| ())?;
            let out_body = http_body::new().map_err(|_| ())?;
            let p = http_req::send_async(out_request, out_body, &backend).map_err(|_| ())?;
            pending.push((label, p));
        }

        let start = Instant::now();
        let mut log = String::from("Sent 2 requests in parallel.\n");

        while !pending.is_empty() {
            let handles: Vec<&async_io::Pollable> = pending.iter().map(|(_, p)| p).collect();
            let ready_index = match async_io::select_with_timeout(&handles, 500) {
                Some(i) => i as usize,
                None => {
                    log.push_str(&format!(
                        "[+{:.2}s] still waiting on {} request(s)\n",
                        start.elapsed().as_secs_f64(),
                        pending.len()
                    ));
                    continue;
                }
            };

            let (label, ready) = pending.remove(ready_index);
            let (response, body) = http_req::await_response(ready).map_err(|_| ())?;
            let status = response.get_status().map_err(|_| ())?;

            let mut buf = Vec::new();
            loop {
                let chunk = http_body::read(&body, 8192).map_err(|_| ())?;
                if chunk.is_empty() {
                    break;
                }
                buf.extend_from_slice(&chunk);
            }

            log.push_str(&format!(
                "[+{:.2}s] \"{label}\" request finished: status {status}, {} bytes\n",
                start.elapsed().as_secs_f64(),
                buf.len()
            ));
        }

        log.push_str(&format!(
            "\nTotal elapsed: {:.2}s\n",
            start.elapsed().as_secs_f64()
        ));

        let response = http_resp::Response::new().map_err(|_| ())?;
        response
            .insert_header("content-type", b"text/plain")
            .map_err(|_| ())?;

        let out_body = http_body::new().map_err(|_| ())?;
        http_body::write(&out_body, log.as_bytes()).map_err(|_| ())?;
        http_resp::send_downstream(response, out_body).map_err(|_| ())?;

        Ok(())
    }
}

bindings::export!(TwoRequestsAtOnce with_types_in bindings);
