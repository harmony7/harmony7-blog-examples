mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{http_body, http_resp},
};

struct ReadingTheBody;

impl http_incoming::Guest for ReadingTheBody {
    fn handle(_request: http_incoming::Request, request_body: http_body::Body) -> Result<(), ()> {
        let ready = request_body.is_ready();
        println!("request body ready before first read: {ready}");

        let mut buf = Vec::new();
        loop {
            let chunk = http_body::read(&request_body, 8192).map_err(|_| ())?;
            if chunk.is_empty() {
                break;
            }
            buf.extend_from_slice(&chunk);
        }
        let text = String::from_utf8_lossy(&buf).into_owned();
        println!("read {} bytes from request body: {text:?}", buf.len());

        let response = http_resp::Response::new().map_err(|_| ())?;
        response
            .insert_header("content-type", b"text/plain")
            .map_err(|_| ())?;

        let out_body = http_body::new().map_err(|_| ())?;
        let msg = format!("You sent {} bytes: {text}\n", buf.len());
        http_body::write(&out_body, msg.as_bytes()).map_err(|_| ())?;

        http_resp::send_downstream(response, out_body).map_err(|_| ())?;

        Ok(())
    }
}

bindings::export!(ReadingTheBody with_types_in bindings);
