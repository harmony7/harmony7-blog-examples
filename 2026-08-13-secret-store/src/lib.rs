mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{http_body, http_req, http_resp, secret_store},
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

struct SecretStoreExample;

impl http_incoming::Guest for SecretStoreExample {
    fn handle(request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let key = get_header_value(&request, "x-key")
            .map_err(|_| ())?
            .unwrap_or_else(|| "api-token".to_string());

        let store = secret_store::Store::open("creds").map_err(|_| ())?;

        // <highlight>
        // get returns a `secret` handle, not the value itself.
        let secret = store.get(&key).map_err(|_| ())?;
        // </highlight>

        let msg = match secret {
            // <highlight>
            // The plaintext bytes only come out via a second call.
            Some(s) => {
                let bytes = s.plaintext(1024).map_err(|_| ())?;
                let value = String::from_utf8_lossy(&bytes).into_owned();
                format!("\"{key}\" = \"{value}\"\n")
            }
            // </highlight>
            None => format!("\"{key}\" is not set in this Secret Store\n"),
        };

        let response = http_resp::Response::new().map_err(|_| ())?;
        response
            .insert_header("content-type", b"text/plain")
            .map_err(|_| ())?;

        let out_body = http_body::new().map_err(|_| ())?;
        http_body::write(&out_body, msg.as_bytes()).map_err(|_| ())?;
        http_resp::send_downstream(response, out_body).map_err(|_| ())?;

        Ok(())
    }
}

bindings::export!(SecretStoreExample with_types_in bindings);
