mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{config_store, http_body, http_req, http_resp},
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

struct ConfigStoreExample;

impl http_incoming::Guest for ConfigStoreExample {
    fn handle(request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let key = get_header_value(&request, "x-key")
            .map_err(|_| ())?
            .unwrap_or_else(|| "greeting".to_string());

        let store = config_store::Store::open("settings").map_err(|_| ())?;
        let value = store.get(&key, 1024).map_err(|_| ())?;

        let msg = match value {
            Some(v) => format!("\"{key}\" = \"{v}\"\n"),
            None => format!("\"{key}\" is not set in this Config Store\n"),
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

bindings::export!(ConfigStoreExample with_types_in bindings);
