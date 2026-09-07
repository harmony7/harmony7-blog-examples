mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_cache, http_resp},
};

struct HttpCacheBeforeTheSend;

fn probe(lines: &mut String, label: &str, options: &http_cache::LookupOptions, req: &http_cache::Request) {
    lines.push_str(&format!("\n--- transaction-lookup with {label} ---\n"));
    match http_cache::Entry::transaction_lookup(req, options) {
        Ok(entry) => {
            lines.push_str("transaction-lookup:   Ok(entry)\n");
            lines.push_str(&format!("state:                {:?}\n", entry.get_state()));
            match entry.get_suggested_backend_request() {
                Ok(r) => {
                    lines.push_str(&format!("suggested method:     {:?}\n", r.get_method(64)));
                    lines.push_str(&format!("suggested uri:        {:?}\n", r.get_uri(256)));
                }
                Err(e) => lines.push_str(&format!("suggested request:    Err({e:?})\n")),
            }
            let _ = http_cache::close_entry(entry);
        }
        Err(e) => lines.push_str(&format!("transaction-lookup:   Err({e:?})\n")),
    }
}

impl http_incoming::Guest for HttpCacheBeforeTheSend {
    fn handle(request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let mut lines = String::new();

        lines.push_str(&format!("is-request-cacheable: {:?}\n", http_cache::is_request_cacheable(&request)));
        lines.push_str(&format!("suggested-cache-key:  {:?}\n", http_cache::get_suggested_cache_key(&request, 64)));

        // <highlight>
        let opened = backend::Backend::open("http_me");
        lines.push_str(&format!("backend open:         {}\n", match &opened {
            Ok(b) => format!("Ok({:?})", b.get_name()),
            Err(e) => format!("Err({e:?})"),
        }));

        probe(&mut lines, "backend: None", &http_cache::LookupOptions {
            override_key: None, backend: None, extra: None,
        }, &request);

        match &opened {
            Ok(b) => probe(&mut lines, "backend: Some(http_me)", &http_cache::LookupOptions {
                override_key: None, backend: Some(b), extra: None,
            }, &request),
            Err(_) => lines.push_str("\n--- transaction-lookup with backend: Some(..) skipped, backend did not open ---\n"),
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

bindings::export!(HttpCacheBeforeTheSend with_types_in bindings);
