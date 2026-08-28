// <fold imports, write_options and lookup_options, unchanged from the insert-and-replace article>
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
// </fold>

// <fold decode, user metadata comes back as raw bytes>
fn decode(v: Result<Option<Vec<u8>>, cache::Error>) -> String {
    match v {
        Ok(Some(bytes)) => format!("{:?}", String::from_utf8_lossy(&bytes)),
        Ok(None) => "Ok(None)".to_string(),
        Err(e) => format!("Err({e:?})"),
    }
}
// </fold>

// <highlight>
fn read_range(entry: &cache::Entry, from: Option<u64>, to: Option<u64>) -> String {
    let options = cache::GetBodyOptions { from, to, extra: None };
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
// </highlight>

struct CoreCacheReadingAHit;

impl http_incoming::Guest for CoreCacheReadingAHit {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let mut lines = String::new();

        // <fold two inserts: one that declares its length and metadata, one that declares neither>
        let declared = b"declared-length".to_vec();
        let mut options = write_options(60_000_000_000);
        options.length = Some(26);
        options.user_metadata = Some(b"written-by=reading-a-hit".to_vec());
        options.stale_while_revalidate_ns = Some(30_000_000_000);
        let writing = cache::insert(&declared, &options).map_err(|_| ())?;
        http_body::write(&writing, b"abcdefghijklmnopqrstuvwxyz").map_err(|_| ())?;
        http_body::close(writing).map_err(|_| ())?;

        let undeclared = b"undeclared-length".to_vec();
        let writing = cache::insert(&undeclared, &write_options(60_000_000_000)).map_err(|_| ())?;
        http_body::write(&writing, b"abcdefghijklmnopqrstuvwxyz").map_err(|_| ())?;
        http_body::close(writing).map_err(|_| ())?;
        // </fold>

        // <highlight>
        // Everything the entry will tell you about a found object.
        let entry = cache::Entry::lookup(&declared, &lookup_options()).map_err(|_| ())?;
        lines.push_str("-- declared length + metadata --\n");
        lines.push_str(&format!("state:      {:?}\n", entry.get_state()));
        lines.push_str(&format!("length:     {:?}\n", entry.get_length()));
        lines.push_str(&format!("max-age-ns: {:?}\n", entry.get_max_age_ns()));
        lines.push_str(&format!("swr-ns:     {:?}\n", entry.get_stale_while_revalidate_ns()));
        lines.push_str(&format!("age-ns:     {:?}\n", entry.get_age_ns()));
        lines.push_str(&format!("metadata:   {:?}\n", decode(entry.get_user_metadata(256))));

        // get-body-options is a byte range, not a whole-object read.
        lines.push_str(&format!("full:       {}\n", read_range(&entry, None, None)));
        lines.push_str(&format!("from 3:     {}\n", read_range(&entry, Some(3), None)));
        lines.push_str(&format!("3..10:      {}\n", read_range(&entry, Some(3), Some(10))));
        lines.push_str(&format!("..5:        {}\n", read_range(&entry, None, Some(5))));
        lines.push_str(&format!("length after reading: {:?}\n", entry.get_length()));
        cache::close_entry(entry).map_err(|_| ())?;

        // The same object, written without declaring a length.
        let entry = cache::Entry::lookup(&undeclared, &lookup_options()).map_err(|_| ())?;
        lines.push_str("\n-- no declared length --\n");
        lines.push_str(&format!("length:     {:?}\n", entry.get_length()));
        lines.push_str(&format!("metadata:   {:?}\n", decode(entry.get_user_metadata(256))));
        lines.push_str(&format!("full:       {}\n", read_range(&entry, None, None)));
        cache::close_entry(entry).map_err(|_| ())?;
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

bindings::export!(CoreCacheReadingAHit with_types_in bindings);
