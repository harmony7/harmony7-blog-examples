// <fold imports, decode and lookup_options, unchanged from the reading-a-hit article>
mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{cache, http_body, http_resp},
};

fn decode(v: Result<Option<Vec<u8>>, cache::Error>) -> String {
    match v {
        Ok(Some(bytes)) => format!("{:?}", String::from_utf8_lossy(&bytes)),
        Ok(None) => "Ok(None)".to_string(),
        Err(e) => format!("Err({e:?})"),
    }
}

fn lookup_options() -> cache::LookupOptions<'static> {
    cache::LookupOptions { request_headers: None, always_use_requested_range: true, extra: None }
}
// </fold>

struct CoreCacheWriteOptions;

impl http_incoming::Guest for CoreCacheWriteOptions {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let mut lines = String::new();
        let key = b"write-options-demo".to_vec();

        // <highlight>
        // Every field the record has, set to something distinguishable.
        let options = cache::WriteOptions {
            max_age_ns: 60_000_000_000,          // the only required field
            request_headers: None,               // non-transactional insert only
            vary_rule: Some("accept-encoding user-agent".to_string()),
            initial_age_ns: Some(5_000_000_000),
            stale_while_revalidate_ns: Some(30_000_000_000),
            surrogate_keys: Some("catalog product-42".to_string()),
            length: Some(11),
            user_metadata: Some(b"origin=example".to_vec()),
            edge_max_age_ns: Some(10_000_000_000),
            sensitive_data: false,
            extra: None,
        };

        let writing = cache::insert(&key, &options).map_err(|_| ())?;
        http_body::write(&writing, b"hello world").map_err(|_| ())?;
        http_body::close(writing).map_err(|_| ())?;

        // Which of them can be read back off the entry?
        let entry = cache::Entry::lookup(&key, &lookup_options()).map_err(|_| ())?;
        lines.push_str(&format!("max-age-ns:  {:?}\n", entry.get_max_age_ns()));
        lines.push_str(&format!("age-ns:      {:?}\n", entry.get_age_ns()));
        lines.push_str(&format!("length:      {:?}\n", entry.get_length()));
        lines.push_str(&format!("metadata:    {}\n", decode(entry.get_user_metadata(256))));
        lines.push_str(&format!("swr-ns:      {:?}\n", entry.get_stale_while_revalidate_ns()));
        cache::close_entry(entry).map_err(|_| ())?;

        // sensitive-data on a second key, to see whether it changes what comes back.
        let secret = b"write-options-sensitive".to_vec();
        let mut options = cache::WriteOptions {
            max_age_ns: 60_000_000_000,
            request_headers: None,
            vary_rule: None,
            initial_age_ns: None,
            stale_while_revalidate_ns: None,
            surrogate_keys: None,
            length: None,
            user_metadata: None,
            edge_max_age_ns: None,
            sensitive_data: true,
            extra: None,
        };
        options.user_metadata = Some(b"secret=yes".to_vec());
        let writing = cache::insert(&secret, &options).map_err(|_| ())?;
        http_body::write(&writing, b"do not log me").map_err(|_| ())?;
        http_body::close(writing).map_err(|_| ())?;

        match cache::Entry::lookup(&secret, &lookup_options()) {
            Ok(entry) => {
                lines.push_str("\n-- sensitive-data: true --\n");
                lines.push_str(&format!("state:    {:?}\n", entry.get_state()));
                lines.push_str(&format!("metadata: {}\n", decode(entry.get_user_metadata(256))));
                let _ = cache::close_entry(entry);
            }
            Err(e) => lines.push_str(&format!("\nsensitive lookup: Err({e:?})\n")),
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

bindings::export!(CoreCacheWriteOptions with_types_in bindings);
