// <fold imports and the option helpers, carried over from the write-options article>
mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{cache, http_body, http_resp, purge},
};
use sha2::{Digest, Sha256};

fn write_options(max_age_ns: u64, surrogate_keys: Option<String>) -> cache::WriteOptions<'static> {
    cache::WriteOptions {
        max_age_ns,
        request_headers: None,
        vary_rule: None,
        initial_age_ns: None,
        stale_while_revalidate_ns: None,
        surrogate_keys,
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

fn state_of(key: &[u8]) -> String {
    match cache::Entry::lookup(&key.to_vec(), &lookup_options()) {
        Ok(entry) => {
            let s = format!("{:?}", entry.get_state());
            let _ = cache::close_entry(entry);
            s
        }
        Err(e) => format!("Err({e:?})"),
    }
}
// </fold>

// <highlight>
/// The technique every Fastly SDK's Simple Cache uses internally: derive a
/// surrogate key from the cache key by hashing it, so the same cache key always
/// produces the same surrogate key at insert time and at purge time.
fn surrogate_key_for(cache_key: &[u8]) -> String {
    let digest = Sha256::digest(cache_key);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}
// </highlight>

struct SurrogateKeyPurging;

impl http_incoming::Guest for SurrogateKeyPurging {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let mut lines = String::new();

        // <highlight>
        // Two objects, tagged with one shared surrogate key and one private each.
        for (key, keys) in [
            (&b"product-42-en"[..], "catalog product-42"),
            (&b"product-42-fr"[..], "catalog product-42"),
            (&b"unrelated"[..], "other"),
        ] {
            let writing = cache::insert(&key.to_vec(), &write_options(60_000_000_000, Some(keys.to_string())))
                .map_err(|_| ())?;
            http_body::write(&writing, key).map_err(|_| ())?;
            http_body::close(writing).map_err(|_| ())?;
        }

        lines.push_str("-- before purge --\n");
        for key in [&b"product-42-en"[..], &b"product-42-fr"[..], &b"unrelated"[..]] {
            lines.push_str(&format!("{:<14} {}\n", String::from_utf8_lossy(key), state_of(key)));
        }

        // One purge, by a key that isn't any object's cache key.
        let options = purge::PurgeOptions { soft_purge: false, extra: None };
        let purged = purge::purge_surrogate_key("product-42", &options);
        lines.push_str(&format!("\npurge-surrogate-key(\"product-42\"): {purged:?}\n"));

        lines.push_str("\n-- after purge --\n");
        for key in [&b"product-42-en"[..], &b"product-42-fr"[..], &b"unrelated"[..]] {
            lines.push_str(&format!("{:<14} {}\n", String::from_utf8_lossy(key), state_of(key)));
        }

        // The verbose form returns a purge id.
        let verbose = purge::purge_surrogate_key_verbose("catalog", &options, 1024);
        lines.push_str(&format!("\nverbose form: {verbose:?}\n"));

        // Deriving a surrogate key from the cache key, so a purge can find it later.
        let key = b"derive-me".to_vec();
        let derived = surrogate_key_for(&key);
        lines.push_str(&format!("\nsha256(\"derive-me\") = {derived}\n"));
        let writing =
            cache::insert(&key, &write_options(60_000_000_000, Some(derived.clone()))).map_err(|_| ())?;
        http_body::write(&writing, b"purgeable by its own key").map_err(|_| ())?;
        http_body::close(writing).map_err(|_| ())?;
        lines.push_str(&format!("before: {}\n", state_of(&key)));
        let purged = purge::purge_surrogate_key(&surrogate_key_for(&key), &options);
        lines.push_str(&format!("purge:  {purged:?}\n"));
        lines.push_str(&format!("after:  {}\n", state_of(&key)));
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

bindings::export!(SurrogateKeyPurging with_types_in bindings);
