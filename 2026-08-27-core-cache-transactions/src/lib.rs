// <fold imports and the helpers carried over from the insert-and-replace article>
mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{cache, http_body, http_resp},
};

fn write_options(max_age_ns: u64) -> cache::WriteOptions<'static> {
    cache::WriteOptions {
        max_age_ns,
        request_headers: None,
        vary_rule: None,
        initial_age_ns: None,
        stale_while_revalidate_ns: None,
        surrogate_keys: None,
        length: None,
        user_metadata: None,
        edge_max_age_ns: None,
        sensitive_data: false,
        extra: None,
    }
}

fn lookup_options() -> cache::LookupOptions<'static> {
    cache::LookupOptions { request_headers: None, always_use_requested_range: false, extra: None }
}

fn read_entry_body(entry: &cache::Entry) -> String {
    let options = cache::GetBodyOptions { from: None, to: None, extra: None };
    let Ok(body) = entry.get_body(&options) else {
        return "<no body>".to_string();
    };
    let mut out = Vec::new();
    while let Ok(chunk) = http_body::read(&body, 1024) {
        if chunk.is_empty() {
            break;
        }
        out.extend_from_slice(&chunk);
    }
    let _ = http_body::close(body);
    String::from_utf8_lossy(&out).into_owned()
}
// </fold>

struct CoreCacheTransactions;

impl http_incoming::Guest for CoreCacheTransactions {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let mut lines = String::new();
        let key = b"transaction-demo".to_vec();

        // <highlight>
        // A transactional lookup on a key nothing has written yet.
        let entry = cache::Entry::transaction_lookup(&key, &lookup_options()).map_err(|_| ())?;
        let state = entry.get_state().map_err(|_| ())?;
        lines.push_str(&format!("first lookup state: {state:?}\n"));
        lines.push_str(&format!("  found:                {}\n", state.contains(cache::LookupState::FOUND)));
        lines.push_str(&format!("  must-insert-or-update: {}\n", state.contains(cache::LookupState::MUST_INSERT_OR_UPDATE)));

        // The state told us we owe the cache an object. Pay the debt.
        let writing = entry.transaction_insert(&write_options(60_000_000_000)).map_err(|_| ())?;
        http_body::write(&writing, b"written under obligation").map_err(|_| ())?;
        http_body::close(writing).map_err(|_| ())?;

        // A second transactional lookup, now that the object exists.
        let again = cache::Entry::transaction_lookup(&key, &lookup_options()).map_err(|_| ())?;
        let state = again.get_state().map_err(|_| ())?;
        lines.push_str(&format!("\nsecond lookup state: {state:?}\n"));
        lines.push_str(&format!("  found:                {}\n", state.contains(cache::LookupState::FOUND)));
        lines.push_str(&format!("  must-insert-or-update: {}\n", state.contains(cache::LookupState::MUST_INSERT_OR_UPDATE)));
        lines.push_str(&format!("  body:                 {}\n", read_entry_body(&again)));
        cache::close_entry(again).map_err(|_| ())?;

        // The async entrypoint hands back a pollable instead of blocking.
        match cache::Entry::transaction_lookup_async(&key, &lookup_options()) {
            Ok(pending) => {
                lines.push_str("\ntransaction-lookup-async: Ok(pending-entry)\n");
                match cache::await_entry(pending) {
                    Ok(awaited) => {
                        lines.push_str(&format!("await-entry state: {:?}\n", awaited.get_state()));
                        let _ = cache::close_entry(awaited);
                    }
                    Err(e) => lines.push_str(&format!("await-entry: Err({e:?})\n")),
                }
            }
            Err(e) => lines.push_str(&format!("\ntransaction-lookup-async: Err({e:?})\n")),
        }
        // </highlight>

        // <fold send the response, unchanged>
        let response = http_resp::Response::new().map_err(|_| ())?;
        response.insert_header("content-type", b"text/plain").map_err(|_| ())?;
        let out_body = http_body::new().map_err(|_| ())?;
        http_body::write(&out_body, lines.as_bytes()).map_err(|_| ())?;
        http_resp::send_downstream(response, out_body).map_err(|_| ())?;
        // </fold>

        Ok(())
    }
}

bindings::export!(CoreCacheTransactions with_types_in bindings);
