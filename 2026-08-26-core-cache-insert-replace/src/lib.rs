mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{cache, http_body, http_resp},
};

// <fold write_options and lookup_options, ten fields and only two of them required>
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
    cache::LookupOptions { request_headers: None, always_use_requested_range: true, extra: None }
}
// </fold>

// <fold read_entry_body, drains an entry's body into a String>
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

struct CoreCacheInsertReplace;

impl http_incoming::Guest for CoreCacheInsertReplace {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let mut lines = String::new();
        let key = b"core-cache-demo".to_vec();

        // <highlight>
        // Write an object in.
        // No transaction, no obligation, no handshake - the write_options parameter is the max-age in nanoseconds
        let writing = cache::insert(&key, &write_options(60_000_000_000)).map_err(|_| ())?;
        http_body::write(&writing, b"first value").map_err(|_| ())?;
        http_body::close(writing).map_err(|_| ())?;

        // Read it back without joining anyone else's request collapsing.
        let found = cache::Entry::lookup(&key, &lookup_options()).map_err(|_| ())?;
        lines.push_str(&format!("state:  {:?}\n", found.get_state()));
        lines.push_str(&format!("body:   {}\n", read_entry_body(&found)));
        lines.push_str(&format!("age-ns: {:?}\n", found.get_age_ns()));
        lines.push_str(&format!("hits:   {:?}\n", found.get_hits()));
        cache::close_entry(found).map_err(|_| ())?;

        // Replace: the other write path, the one that can read what it's overwriting.
        let replace_options = cache::ReplaceOptions {
            request_headers: None,
            replace_strategy: Some(cache::ReplaceStrategy::Immediate),
            always_use_requested_range: true,
            extra: None,
        };
        match cache::ReplaceEntry::replace(&key, &replace_options) {
            Ok(replacing) => {
                lines.push_str(&format!("\nreplace state:  {:?}\n", replacing.get_state()));
                let writing = cache::replace_insert(replacing, &write_options(60_000_000_000))
                    .map_err(|_| ())?;
                http_body::write(&writing, b"second value").map_err(|_| ())?;
                http_body::close(writing).map_err(|_| ())?;

                let after = cache::Entry::lookup(&key, &lookup_options()).map_err(|_| ())?;
                lines.push_str(&format!("after replace: {}\n", read_entry_body(&after)));
                cache::close_entry(after).map_err(|_| ())?;
            }
            Err(e) => lines.push_str(&format!("\nreplace: Err({e:?})\n")),
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

bindings::export!(CoreCacheInsertReplace with_types_in bindings);
