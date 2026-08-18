mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{device_detection, http_body, http_req, http_resp},
};

// <fold get_header_value, unchanged from the response article>
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
// </fold>

// <highlight>
fn lookup(user_agent: &str) -> Result<Option<String>, device_detection::Error> {
    let mut max_len: u64 = 1024;
    loop {
        match device_detection::lookup(user_agent, max_len) {
            Ok(Some(json)) => return Ok(Some(json)),
            Ok(None) => return Ok(None),
            Err(device_detection::Error::BufferLen(needed)) => max_len = needed,
            Err(e) => return Err(e),
        }
    }
}
// </highlight>

struct DeviceDetectionExample;

impl http_incoming::Guest for DeviceDetectionExample {
    fn handle(request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let user_agent = get_header_value(&request, "user-agent")
            .map_err(|_| ())?
            .unwrap_or_default();

        let msg = match lookup(&user_agent).map_err(|_| ())? {
            Some(json) => format!("User-Agent: {user_agent}\n\n{json}\n"),
            None => format!("User-Agent: {user_agent}\n\nNo device data for this User-Agent.\n"),
        };

        // <fold send the response, unchanged>
        let response = http_resp::Response::new().map_err(|_| ())?;
        response
            .insert_header("content-type", b"text/plain")
            .map_err(|_| ())?;

        let out_body = http_body::new().map_err(|_| ())?;
        http_body::write(&out_body, msg.as_bytes()).map_err(|_| ())?;
        http_resp::send_downstream(response, out_body).map_err(|_| ())?;
        // </fold>

        Ok(())
    }
}

bindings::export!(DeviceDetectionExample with_types_in bindings);
