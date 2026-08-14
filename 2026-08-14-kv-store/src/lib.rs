mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{http_body, http_req, http_resp, kv_store},
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

// <fold reading a body to a String, the same loop as the body article>
fn read_body_to_string(body: &http_body::Body) -> Result<String, http_body::Error> {
    let mut buf = Vec::new();
    loop {
        let chunk = http_body::read(body, 8192)?;
        if chunk.is_empty() {
            break;
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(String::from_utf8_lossy(&buf).into_owned())
}
// </fold>

struct KvStoreExample;

impl http_incoming::Guest for KvStoreExample {
    fn handle(request: http_incoming::Request, request_body: http_body::Body) -> Result<(), ()> {
        let key = get_header_value(&request, "x-key")
            .map_err(|_| ())?
            .unwrap_or_else(|| "demo-note".to_string());

        let incoming_value = read_body_to_string(&request_body).map_err(|_| ())?;
        let value = if incoming_value.is_empty() {
            "Hello from the KV Store!".to_string()
        } else {
            incoming_value
        };

        let store = kv_store::Store::open("notes").map_err(|_| ())?;
        let mut log = String::new();

        // insert-options is a plain record, not a builder resource like
        // dynamic-backend-options was — the whole thing is one value.
        let insert_body = http_body::new().map_err(|_| ())?;
        http_body::write(&insert_body, value.as_bytes()).map_err(|_| ())?;
        let insert_options = kv_store::InsertOptions {
            background_fetch: false,
            if_generation_match: None,
            metadata: Some("written by the kv-store example".to_string()),
            time_to_live_sec: None,
            mode: kv_store::InsertMode::Overwrite,
            extra: None,
        };
        // <highlight>
        let pending_insert = store
            .insert_async(&key, insert_body, &insert_options)
            .map_err(|_| ())?;
        kv_store::await_insert(pending_insert).map_err(|_| ())?;
        // </highlight>
        log.push_str(&format!("inserted \"{key}\" = \"{value}\"\n"));

        // <highlight>
        // lookup hands back an entry, not the value directly.
        let pending_lookup = store.lookup_async(&key).map_err(|_| ())?;
        let entry = kv_store::await_lookup(pending_lookup)
            .map_err(|_| ())?
            .ok_or(())?;
        // </highlight>
        let generation = entry.generation();
        let metadata = entry.metadata(256).map_err(|_| ())?;
        let entry_body = entry.take_body().ok_or(())?;
        let read_back = read_body_to_string(&entry_body).map_err(|_| ())?;
        log.push_str(&format!(
            "looked up \"{key}\": value = \"{read_back}\", generation = {generation}, metadata = {metadata:?}\n"
        ));

        let pending_delete = store.delete_async(&key).map_err(|_| ())?;
        let deleted = kv_store::await_delete(pending_delete).map_err(|_| ())?;
        log.push_str(&format!("deleted \"{key}\": {deleted}\n"));

        let pending_lookup_2 = store.lookup_async(&key).map_err(|_| ())?;
        let after_delete = kv_store::await_lookup(pending_lookup_2).map_err(|_| ())?;
        log.push_str(&format!(
            "looked up \"{key}\" again: {}\n",
            match after_delete {
                Some(_) => "still there",
                None => "gone",
            }
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

bindings::export!(KvStoreExample with_types_in bindings);
