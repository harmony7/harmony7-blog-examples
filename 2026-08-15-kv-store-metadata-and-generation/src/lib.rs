mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{http_body, http_resp, kv_store},
};

// <fold reading a body to a String, the same loop as the previous KV Store article>
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

struct KvStoreMetadataAndGenerationExample;

impl http_incoming::Guest for KvStoreMetadataAndGenerationExample {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let key = "beta-checkout";
        let store = kv_store::Store::open("flags").map_err(|_| ())?;
        let mut log = String::new();

        // First write: nothing to compare a generation against yet.
        let seed_body = http_body::new().map_err(|_| ())?;
        http_body::write(&seed_body, b"off").map_err(|_| ())?;
        let seed_options = kv_store::InsertOptions {
            background_fetch: false,
            if_generation_match: None,
            metadata: Some("note=seeded off".to_string()),
            time_to_live_sec: None,
            mode: kv_store::InsertMode::Overwrite,
            extra: None,
        };
        let pending = store
            .insert_async(key, seed_body, &seed_options)
            .map_err(|_| ())?;
        kv_store::await_insert(pending).map_err(|_| ())?;
        log.push_str(&format!("seeded \"{key}\" = \"off\"\n"));

        // <highlight>
        // Read metadata and generation without ever calling take-body.
        let pending_lookup = store.lookup_async(key).map_err(|_| ())?;
        let entry = kv_store::await_lookup(pending_lookup)
            .map_err(|_| ())?
            .ok_or(())?;
        let metadata = entry.metadata(256).map_err(|_| ())?;
        let generation = entry.generation();
        // </highlight>
        log.push_str(&format!(
            "read metadata = {metadata:?}, generation = {generation}, without touching the body\n"
        ));

        // A correct conditional write: the generation we just read still matches the store's.
        let update_body = http_body::new().map_err(|_| ())?;
        http_body::write(&update_body, b"on").map_err(|_| ())?;
        let update_options = kv_store::InsertOptions {
            background_fetch: false,
            // <highlight>
            if_generation_match: Some(generation),
            // </highlight>
            metadata: Some("note=turned on".to_string()),
            time_to_live_sec: None,
            mode: kv_store::InsertMode::Overwrite,
            extra: None,
        };
        let pending = store
            .insert_async(key, update_body, &update_options)
            .map_err(|_| ())?;
        match kv_store::await_insert(pending) {
            Ok(()) => log.push_str("conditional write with the current generation: succeeded\n"),
            Err(e) => log.push_str(&format!(
                "conditional write with the current generation: unexpectedly failed: {e:?}\n"
            )),
        }

        // A conflicting write: reuse the generation from before that last successful write,
        // the way a second, slower request that read the entry at the same time we did would.
        let stale_body = http_body::new().map_err(|_| ())?;
        http_body::write(&stale_body, b"on-again").map_err(|_| ())?;
        let stale_options = kv_store::InsertOptions {
            background_fetch: false,
            // <highlight>
            if_generation_match: Some(generation),
            // </highlight>
            metadata: Some("note=should not land".to_string()),
            time_to_live_sec: None,
            mode: kv_store::InsertMode::Overwrite,
            extra: None,
        };
        let pending = store
            .insert_async(key, stale_body, &stale_options)
            .map_err(|_| ())?;
        // <highlight>
        match kv_store::await_insert(pending) {
            Ok(()) => log.push_str("conditional write with the stale generation: unexpectedly succeeded\n"),
            Err(kv_store::KvError::PreconditionFailed) => log.push_str(
                "conditional write with the stale generation: rejected, precondition-failed\n",
            ),
            Err(e) => log.push_str(&format!(
                "conditional write with the stale generation: failed with {e:?}\n"
            )),
        }
        // </highlight>

        // Confirm which write actually won, this time reading the body too.
        let pending_lookup = store.lookup_async(key).map_err(|_| ())?;
        let final_entry = kv_store::await_lookup(pending_lookup)
            .map_err(|_| ())?
            .ok_or(())?;
        let final_generation = final_entry.generation();
        let final_body = final_entry.take_body().ok_or(())?;
        let final_value = read_body_to_string(&final_body).map_err(|_| ())?;
        log.push_str(&format!(
            "final value of \"{key}\" = \"{final_value}\", generation = {final_generation}\n"
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

bindings::export!(KvStoreMetadataAndGenerationExample with_types_in bindings);
