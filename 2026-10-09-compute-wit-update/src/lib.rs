mod bindings;

use std::time::Instant;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{async_io, backend, http_body, http_req, http_resp, log},
};

fn respond(text: &str) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    http_resp::send_downstream(response, body).map_err(|_| ())
}

/// Clones a log endpoint, drops the original, and writes through the clone.
fn clone_log() -> Result<(), ()> {
    let endpoint = log::Endpoint::open("my_endpoint").map_err(|_| ())?;
    let copy = endpoint.clone();
    drop(endpoint);
    copy.write(b"written through a clone");
    respond("log endpoint: wrote through the clone after dropping the original\n")
}

/// Clones a backend, drops the original, and sends through the clone.
fn clone_backend() -> Result<(), ()> {
    let original = backend::Backend::open("http_me").map_err(|_| ())?;
    let copy = original.clone();
    drop(original);

    let request = http_req::Request::new().map_err(|_| ())?;
    request.set_uri("https://http-me.fastly.dev/anything").map_err(|_| ())?;
    let result = http_req::send_uncached(request, http_body::new().map_err(|_| ())?, &copy);
    let status = result.map(|(response, _)| response.get_status());
    respond(&format!(
        "backend clone: name {:?}, send through it: {status:?}\n",
        copy.get_name()
    ))
}

fn query_param<'a>(uri: &'a str, name: &str) -> Option<&'a str> {
    let query = uri.split_once('?')?.1;
    query
        .split('&')
        .find_map(|pair| pair.strip_prefix(name)?.strip_prefix('='))
}

/// Starts a cacheable request for a slow URL, optionally with a cache
/// lookup timeout set through extra-cache-override-details.
fn start(
    backend: &backend::Backend,
    uri: &str,
    lookup_timeout: Option<u32>,
) -> Result<http_req::PendingResponse, ()> {
    let request = http_req::Request::new().map_err(|_| ())?;
    request.set_uri(uri).map_err(|_| ())?;

    if let Some(ms) = lookup_timeout {
        let extra = http_req::ExtraCacheOverrideDetails::new();
        extra.set_lookup_timeout(ms);
        request
            .set_cache_override(&http_req::CacheOverride::Override(
                http_req::CacheOverrideDetails {
                    ttl: None,
                    stale_while_revalidate: None,
                    pci: false,
                    surrogate_key: None,
                    extra: Some(&extra),
                },
            ))
            .map_err(|_| ())?;
    }

    http_req::send_async(request, http_body::new().map_err(|_| ())?, backend).map_err(|_| ())
}

fn header(response: &http_resp::Response, name: &str) -> String {
    match response.get_header_value(name, 256) {
        Ok(Some(v)) => String::from_utf8_lossy(&v).into_owned(),
        _ => "-".to_string(),
    }
}

/// Sends two requests for the same slow, cacheable URL at once, so the
/// second can collapse onto the first. Only the second gets the timeout.
fn collapse(incoming_uri: &str) -> Result<(), ()> {
    let run = query_param(incoming_uri, "run").unwrap_or("default");
    let timeout = query_param(incoming_uri, "timeout").and_then(|t| t.parse().ok());
    let uri = format!("https://http-me.fastly.dev/anything?wait=2000&cache=60&run={run}");

    let backend = backend::Backend::open("http_me").map_err(|_| ())?;
    let started = Instant::now();
    let mut pending = vec![
        ("first", start(&backend, &uri, None)?),
        ("second", start(&backend, &uri, timeout)?),
    ];

    let mut report = format!("run {run}, lookup timeout on the second request: {timeout:?} ms\n");
    while !pending.is_empty() {
        let handles: Vec<&http_req::PendingResponse> = pending.iter().map(|(_, p)| p).collect();
        let ready = async_io::select(&handles) as usize;
        let (label, handle) = pending.remove(ready);
        let ms = started.elapsed().as_millis();
        let outcome = match http_req::await_response(handle) {
            Ok((response, _)) => format!(
                "status {:?}, x-cache {}",
                response.get_status(),
                header(&response, "x-cache")
            ),
            Err(e) => format!("{:?}, detail {:?}", e.error, e.detail),
        };
        report.push_str(&format!("{label:<6} finished at {ms:>5} ms: {outcome}\n"));
    }
    respond(&report)
}

struct ComputeWitUpdate;

impl http_incoming::Guest for ComputeWitUpdate {
    fn handle(request: http_incoming::Request, _body: http_body::Body) -> Result<(), ()> {
        let uri = request.get_uri(1024).map_err(|_| ())?;
        let path = uri.split('?').next().unwrap_or("");
        let path = path.rsplit_once('/').map(|(_, p)| p).unwrap_or("");

        match path {
            "clone-log" => clone_log(),
            "clone-backend" => clone_backend(),
            "collapse" => collapse(&uri),
            _ => respond("try /clone-log, /clone-backend, or /collapse?run=<anything>&timeout=<ms>\n"),
        }
    }
}

bindings::export!(ComputeWitUpdate with_types_in bindings);
