mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_cache, http_req, http_resp},
};

struct HttpCacheAfterTheSend;

// <fold helpers: drain a body, read one header as a string>
fn drain(body: &http_body::Body) -> usize {
    let mut n = 0;
    loop {
        match http_body::read(body, 8192) {
            Ok(chunk) if chunk.is_empty() => break,
            Ok(chunk) => n += chunk.len(),
            Err(_) => break,
        }
    }
    n
}

fn hdr(resp: &http_resp::Response, name: &str) -> String {
    match resp.get_header_value(name, 256) {
        Ok(Some(v)) => String::from_utf8_lossy(&v).into_owned(),
        Ok(None) => "-".to_string(),
        Err(e) => format!("err {e:?}"),
    }
}

// </fold>

fn run(log: &mut String, request: &http_incoming::Request) -> Result<(), String> {
    let be = backend::Backend::open("http_me").map_err(|e| format!("backend open: {e:?}"))?;

    // Any URL carrying "ck=" shares one fixed cache key, so a response stored
    // under one path can be revalidated by a request to a different one.
    let uri = request.get_uri(1024).unwrap_or_default();
    let shared = uri.contains("ck=");
    let override_key = shared.then(|| vec![0x5au8; 32]);
    log.push_str(&format!("override-key:              {}\n", if shared { "shared 32-byte key" } else { "none (derived)" }));

    let entry = http_cache::Entry::transaction_lookup(
        request,
        &http_cache::LookupOptions { override_key, backend: Some(&be), extra: None },
    )
    .map_err(|e| format!("transaction-lookup: {e:?}"))?;

    let state = entry.get_state().map_err(|e| format!("get-state: {e:?}"))?;
    log.push_str(&format!("lookup state:              {state:?}\n"));

    if !state.contains(http_cache::LookupState::MUST_INSERT_OR_UPDATE) {
        log.push_str("no obligation; serving what is stored\n");
        match entry.get_found_response(0) {
            Ok(Some((resp, body))) => {
                log.push_str(&format!("found status:              {:?}\n", resp.get_status()));
                log.push_str(&format!("found body bytes:          {}\n", drain(&body)));
            }
            Ok(None) => log.push_str("found response:            none\n"),
            Err(e) => log.push_str(&format!("get-found-response:        Err({e:?})\n")),
        }
        let _ = http_cache::close_entry(entry);
        return Ok(());
    }

    let backend_req = entry
        .get_suggested_backend_request()
        .map_err(|e| format!("get-suggested-backend-request: {e:?}"))?;
    let empty = http_body::new().map_err(|e| format!("body new: {e:?}"))?;

    let (resp, resp_body) = http_req::send_uncached(backend_req, empty, &be)
        .map_err(|e| format!("send-uncached: {e:?}"))?;
    log.push_str(&format!("origin status:             {:?}\n", resp.get_status()));
    log.push_str(&format!("origin cache-control:      {}\n", hdr(&resp, "cache-control")));
    log.push_str(&format!("origin etag:               {}\n", hdr(&resp, "etag")));

    // <highlight>
    let (action, adjusted) = entry
        .prepare_response_for_storage(&resp)
        .map_err(|e| format!("prepare-response-for-storage: {e:?}"))?;
    log.push_str(&format!("\nstorage-action:            {action:?}\n"));
    log.push_str(&format!("adjusted status:           {:?}\n", adjusted.get_status()));

    let sw = entry
        .get_suggested_write_options(&adjusted)
        .map_err(|e| format!("get-suggested-write-options: {e:?}"))?;
    let opts = http_cache::WriteOptions {
        max_age_ns: sw.get_max_age_ns(),
        vary_rule: sw.get_vary_rule(256).ok(),
        initial_age_ns: Some(sw.get_initial_age_ns()),
        stale_while_revalidate_ns: Some(sw.get_stale_while_revalidate_ns()),
        stale_if_error_ns: Some(sw.get_stale_if_error_ns()),
        surrogate_keys: sw.get_surrogate_keys(256).ok(),
        length: sw.get_length(),
        sensitive_data: sw.get_sensitive_data(),
        extra: None,
    };
    log.push_str(&format!("suggested max-age-ns:      {}\n", opts.max_age_ns));
    log.push_str(&format!("suggested swr-ns:          {:?}\n", opts.stale_while_revalidate_ns));
    log.push_str(&format!("suggested sie-ns:          {:?}\n", opts.stale_if_error_ns));
    log.push_str(&format!("suggested vary-rule:       {:?}\n", opts.vary_rule));

    match action {
        http_cache::StorageAction::Insert => {
            let out = entry
                .transaction_insert(adjusted, &opts)
                .map_err(|e| format!("transaction-insert: {e:?}"))?;
            let mut n = 0;
            loop {
                match http_body::read(&resp_body, 8192) {
                    Ok(c) if c.is_empty() => break,
                    Ok(c) => {
                        n += c.len();
                        let _ = http_body::write(&out, &c);
                    }
                    Err(_) => break,
                }
            }
            let _ = http_body::close(out);
            log.push_str(&format!("discharged with:           transaction-insert ({n} bytes)\n"));
        }
        http_cache::StorageAction::Update => {
            entry
                .transaction_update(adjusted, &opts)
                .map_err(|e| format!("transaction-update: {e:?}"))?;
            log.push_str("discharged with:           transaction-update\n");
        }
        http_cache::StorageAction::DoNotStore => {
            entry
                .transaction_abandon()
                .map_err(|e| format!("transaction-abandon: {e:?}"))?;
            log.push_str("discharged with:           transaction-abandon\n");
        }
        http_cache::StorageAction::RecordUncacheable => {
            entry
                .transaction_record_not_cacheable(&opts)
                .map_err(|e| format!("transaction-record-not-cacheable: {e:?}"))?;
            log.push_str("discharged with:           transaction-record-not-cacheable\n");
        }
    }

    // </highlight>

    Ok(())
}

// <fold send the log downstream as the response>
impl http_incoming::Guest for HttpCacheAfterTheSend {
    fn handle(request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let mut log = String::new();
        log.push_str(&format!("uri:                       {:?}\n", request.get_uri(256)));
        if let Err(e) = run(&mut log, &request) {
            log.push_str(&format!("\nFAILED: {e}\n"));
        }
        let response = http_resp::Response::new().map_err(|_| ())?;
        response.insert_header("content-type", b"text/plain").map_err(|_| ())?;
        let out_body = http_body::new().map_err(|_| ())?;
        http_body::write(&out_body, log.as_bytes()).map_err(|_| ())?;
        http_resp::send_downstream(response, out_body).map_err(|_| ())?;
        Ok(())
    }
}

// </fold>
bindings::export!(HttpCacheAfterTheSend with_types_in bindings);
