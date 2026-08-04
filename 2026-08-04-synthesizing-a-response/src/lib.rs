mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{http_body, http_req, http_resp},
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

/// Reads every value for a request header, splitting the NUL-delimited
/// buffer the ABI hands back and paging through `cursor` until it says
/// there's nothing left.
fn get_header_values(
    request: &http_incoming::Request,
    name: &str,
) -> Result<Vec<String>, http_req::Error> {
    let mut values = Vec::new();
    let mut cursor: u32 = 0;
    let mut max_len: u64 = 256;

    loop {
        match request.get_header_values(name, max_len, cursor) {
            Ok((bytes, more)) => {
                values.extend(
                    bytes
                        .split(|&b| b == 0)
                        .filter(|chunk| !chunk.is_empty())
                        .map(|chunk| String::from_utf8_lossy(chunk).into_owned()),
                );
                match more {
                    Some(next_cursor) => cursor = next_cursor,
                    None => return Ok(values),
                }
            }
            Err(http_req::Error::BufferLen(needed)) => max_len = needed,
            Err(e) => return Err(e),
        }
    }
}

struct SimpleResponse;

impl http_incoming::Guest for SimpleResponse {
    fn handle(request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let user_agent = get_header_value(&request, "user-agent")
            .map_err(|_| ())?
            .unwrap_or_else(|| "(none)".to_string());
        let accept_values = get_header_values(&request, "accept").map_err(|_| ())?;

        let response = http_resp::Response::new().map_err(|_| ())?;
        response
            .insert_header("content-type", b"text/plain")
            .map_err(|_| ())?;

        let body = http_body::new().map_err(|_| ())?;
        let msg =
            format!("Hello, world! Your user-agent is: {user_agent}\nAccept: {accept_values:?}\n");
        http_body::write(&body, msg.as_bytes()).map_err(|_| ())?;

        http_resp::send_downstream(response, body).map_err(|_| ())?;

        Ok(())
    }
}

bindings::export!(SimpleResponse with_types_in bindings);
