// <fold imports and the drain helper, carried over from earlier articles>
mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{cache, compute_runtime, http_body, http_resp, purge},
};
use sha2::{Digest, Sha256};
use std::cell::Cell;

fn lookup_options() -> cache::LookupOptions<'static> {
    cache::LookupOptions { request_headers: None, always_use_requested_range: true, extra: None }
}

fn drain(body: &http_body::Body) -> Vec<u8> {
    let mut out = Vec::new();
    while let Ok(chunk) = http_body::read(body, 1024) {
        if chunk.is_empty() {
            break;
        }
        out.extend_from_slice(&chunk);
    }
    out
}

/// Reads a found entry's body to the end. Both operations below need it.
fn read_body(entry: &cache::Entry) -> Option<Vec<u8>> {
    let options = cache::GetBodyOptions { from: None, to: None, extra: None };
    let body = entry.get_body(&options).ok()?;
    let out = drain(&body);
    let _ = http_body::close(body);
    Some(out)
}
// </fold>

// <highlight>
/// Every SDK's Simple Cache derives *two* surrogate keys from one cache key, so
/// that `purge` can find the object again without anything being tracked in
/// between. The POP-scoped one folds this POP's name into the hash; the global
/// one hashes the key alone. Uppercase hex, because that is what the SDKs emit,
/// and a digest that doesn't match theirs byte for byte purges nothing.
enum PurgeScope {
    Pop,
    Global,
}

fn surrogate_key_for(key: &[u8], scope: PurgeScope) -> String {
    let mut sha = Sha256::new();
    sha.update(key);
    if let PurgeScope::Pop = scope {
        sha.update(compute_runtime::get_pop());
    }
    sha.finalize().iter().map(|b| format!("{b:02X}")).collect()
}
// </highlight>

/// `get`: hand back the cached bytes, or nothing at all. Non-collapsing, on
/// purpose: a miss is a miss, and nobody is elected to do anything about it.
fn simple_get(key: &[u8]) -> Option<Vec<u8>> {
    let entry = cache::Entry::lookup(key, &lookup_options()).ok()?;
    let state = entry.get_state().ok()?;
    let bytes =
        if state.contains(cache::LookupState::FOUND) { read_body(&entry) } else { None };
    let _ = cache::close_entry(entry);
    bytes
}

// <highlight>
/// `get_or_set`: a *transaction*, not a lookup with an insert bolted on. The
/// value arrives as a closure, and the closure only runs if the cache elects
/// this instance to produce the object.
fn simple_get_or_set<F>(key: &[u8], ttl_ns: u64, fill: F) -> Result<Vec<u8>, cache::Error>
where
    F: FnOnce() -> Vec<u8>,
{
    // Collapsing lookup: either the object is already here, or we get the job.
    let entry = cache::Entry::transaction_lookup(key, &lookup_options())?;
    let state = entry.get_state()?;

    if !state.contains(cache::LookupState::MUST_INSERT_OR_UPDATE) {
        // Someone else produced it, possibly while we were waiting. Read theirs.
        let found = read_body(&entry).ok_or(cache::Error::GenericError);
        let _ = cache::close_entry(entry);
        return found;
    }

    // Elected. Only now is the value worth producing.
    let value = fill();
    let options = cache::WriteOptions {
        max_age_ns: ttl_ns,
        request_headers: None,
        vary_rule: None,
        initial_age_ns: None,
        stale_while_revalidate_ns: None,
        // The space-delimited field earns its shape here: two tags, one object.
        surrogate_keys: Some(format!(
            "{} {}",
            surrogate_key_for(key, PurgeScope::Pop),
            surrogate_key_for(key, PurgeScope::Global),
        )),
        length: None,
        user_metadata: None,
        edge_max_age_ns: None,
        sensitive_data: false,
        extra: None,
    };

    // Write and read back at once, so every instance collapsed behind us is
    // released as the bytes land rather than after they finish.
    let (writing, reading) = entry.transaction_insert_and_stream_back(&options)?;
    http_body::write(&writing, &value).map_err(|_| cache::Error::GenericError)?;
    http_body::close(writing).map_err(|_| cache::Error::GenericError)?;
    let stored = read_body(&reading).ok_or(cache::Error::GenericError);
    let _ = cache::close_entry(reading);
    stored
}
// </highlight>

/// `purge`: reachable only because `get_or_set` tagged the object on the way in.
/// POP scope is the default, matching the SDKs; global is the opt-in.
fn simple_purge(key: &[u8], scope: PurgeScope) -> Result<(), cache::Error> {
    let options = purge::PurgeOptions { soft_purge: false, extra: None };
    purge::purge_surrogate_key(&surrogate_key_for(key, scope), &options)
}

struct SimpleCacheByHand;

impl http_incoming::Guest for SimpleCacheByHand {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let mut lines = String::new();
        let key = b"greeting";

        // <highlight>
        let show = |v: Option<Vec<u8>>| match v {
            Some(bytes) => format!("Some({:?})", String::from_utf8_lossy(&bytes)),
            None => "None".to_string(),
        };

        // Counts how many times a fill closure was run.
        let fills = Cell::new(0);

        lines.push_str(&format!("get before anything:  {}\n", show(simple_get(key))));

        let first = simple_get_or_set(key, 60_000_000_000, || {
            fills.set(fills.get() + 1);
            b"hello".to_vec()
        })
        .map_err(|_| ())?;
        lines.push_str(&format!(
            "get_or_set (elected): {:<12} fills run: {}\n",
            format!("{:?}", String::from_utf8_lossy(&first)),
            fills.get()
        ));

        // The second call isn't elected, so the closure is never run at all: the
        // value it would have written is never even produced.
        let second = simple_get_or_set(key, 60_000_000_000, || {
            fills.set(fills.get() + 1);
            b"REPLACED".to_vec()
        })
        .map_err(|_| ())?;
        lines.push_str(&format!(
            "get_or_set (found):   {:<12} fills run: {}\n",
            format!("{:?}", String::from_utf8_lossy(&second)),
            fills.get()
        ));

        lines.push_str(&format!("get after set:        {}\n", show(simple_get(key))));

        lines.push_str(&format!("\npop:        {:?}\n", compute_runtime::get_pop()));
        lines.push_str(&format!("pop key:    {}\n", surrogate_key_for(key, PurgeScope::Pop)));
        lines.push_str(&format!("global key: {}\n", surrogate_key_for(key, PurgeScope::Global)));

        lines.push_str(&format!("\npurge (pop scope): {:?}\n", simple_purge(key, PurgeScope::Pop)));
        lines.push_str(&format!("get after purge:      {}\n", show(simple_get(key))));

        // Nothing is cached now, so the same rejected closure gets elected.
        let third = simple_get_or_set(key, 60_000_000_000, || {
            fills.set(fills.get() + 1);
            b"REPLACED".to_vec()
        })
        .map_err(|_| ())?;
        lines.push_str(&format!(
            "get_or_set (elected): {:<12} fills run: {}\n",
            format!("{:?}", String::from_utf8_lossy(&third)),
            fills.get()
        ));
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

bindings::export!(SimpleCacheByHand with_types_in bindings);
