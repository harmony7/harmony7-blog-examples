mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_req, http_resp},
};
use std::thread::sleep;
use std::time::{Duration, Instant};

struct StreamingARequest;

impl http_incoming::Guest for StreamingARequest {
    fn handle(_request: http_incoming::Request, request_body: http_body::Body) -> Result<(), ()> {
        let backend = backend::Backend::open("http_me").map_err(|_| ())?;

        let out_request = http_req::Request::new().map_err(|_| ())?;
        out_request.set_method("POST").map_err(|_| ())?;
        out_request.set_uri("/anything").map_err(|_| ())?;

        let out_body = http_body::new().map_err(|_| ())?;

        let start = Instant::now();
        let mut log = String::new();

        // The request begins sending immediately, with out_body still empty.
        // pending doesn't wait for a single byte of it.
        let pending =
            http_req::send_async_streaming(out_request, &out_body, &backend).map_err(|_| ())?;
        log.push_str(&format!(
            "[+{:.2}s] send-async-streaming returned a pending-response; out_body has nothing written yet\n",
            start.elapsed().as_secs_f64()
        ));

        // Read the incoming body in small pieces and forward each one to
        // out_body as soon as it shows up, rather than buffering the whole
        // thing first. The artificial sleep stands in for a client trickling
        // a large upload in slowly.
        loop {
            let chunk = http_body::read(&request_body, 8).map_err(|_| ())?;
            if chunk.is_empty() {
                break;
            }
            http_body::write(&out_body, &chunk).map_err(|_| ())?;
            sleep(Duration::from_millis(250));
            log.push_str(&format!(
                "[+{:.2}s] forwarded {} byte(s): {:?}\n",
                start.elapsed().as_secs_f64(),
                chunk.len(),
                String::from_utf8_lossy(&chunk)
            ));
        }

        // A successful stream termination, not just dropping the handle.
        http_body::close(out_body).map_err(|_| ())?;
        log.push_str(&format!(
            "[+{:.2}s] closed out_body\n",
            start.elapsed().as_secs_f64()
        ));

        // Doesn't resolve until the streamed body above is actually finished.
        let (backend_response, backend_body) =
            http_req::await_response(pending).map_err(|_| ())?;
        let status = backend_response.get_status().map_err(|_| ())?;
        log.push_str(&format!(
            "[+{:.2}s] await-response returned: status {status}\n",
            start.elapsed().as_secs_f64()
        ));

        let mut buf = Vec::new();
        loop {
            let chunk = http_body::read(&backend_body, 8192).map_err(|_| ())?;
            if chunk.is_empty() {
                break;
            }
            buf.extend_from_slice(&chunk);
        }
        let backend_text = String::from_utf8_lossy(&buf).into_owned();

        log.push_str(&format!("\nBackend said:\n{backend_text}\n"));

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

bindings::export!(StreamingARequest with_types_in bindings);
