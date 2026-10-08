mod bindings;

use std::time::{Duration, Instant};

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_req, http_resp},
};

fn respond(text: &str) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    http_resp::send_downstream(response, body).map_err(|_| ())
}

/// Tries each health check setter with a value its doc comment rules out.
fn rules() -> Result<(), ()> {
    let mut lines = Vec::new();
    let mut check = |label: &str, f: &dyn Fn(&backend::HealthcheckOptions) -> Result<(), backend::Error>| {
        let h = backend::HealthcheckOptions::new("http-me.fastly.dev").expect("new");
        lines.push(format!("{label:<44} {:?}", f(&h)));
    };
    check("method(\"GE T\")", &|h| h.method("GE T"));
    check("window(2), threshold still 3", &|h| h.window(2));
    check("window(16)", &|h| h.window(16));
    check("threshold(6), window still 5", &|h| h.threshold(6));
    check("initial(6), window still 5", &|h| h.initial(6));
    check("interval-ms(500)", &|h| h.interval_ms(500));
    check("interval-ms(4_000), timeout still 5_000", &|h| h.interval_ms(4_000));
    check("interval-ms(10_000), timeout still 5_000", &|h| h.interval_ms(10_000));
    check("timeout-ms(1_000), interval still 15_000", &|h| h.timeout_ms(1_000));
    respond(&(lines.join("\n") + "\n"))
}

/// A health-checked backend whose check hits the given http-me path.
fn checked(name: &str, path: &str) -> Result<backend::Backend, String> {
    let h = backend::HealthcheckOptions::new("http-me.fastly.dev").map_err(|e| format!("{e:?}"))?;
    h.path(path).map_err(|e| format!("path: {e:?}"))?;
    // The window can't shrink below the threshold or the initial count,
    // so those come down first.
    h.threshold(1).map_err(|e| format!("threshold: {e:?}"))?;
    h.initial(0).map_err(|e| format!("initial: {e:?}"))?;
    h.window(2).map_err(|e| format!("window: {e:?}"))?;
    h.interval_ms(1_000).map_err(|e| format!("interval: {e:?}"))?;
    h.timeout_ms(1_000).map_err(|e| format!("timeout: {e:?}"))?;

    let o = backend::DynamicBackendOptions::new();
    o.use_tls(true);
    o.healthcheck(h);
    backend::register_dynamic_backend(name, "http-me.fastly.dev", o).map_err(|e| format!("{e:?}"))
}

/// Watches two health-checked backends for a few seconds, then sends a
/// request through each.
fn health() -> Result<(), ()> {
    let mut report = String::new();
    let good = checked("good", "/status=200");
    let bad = checked("bad", "/status=503");
    let (good, bad) = match (good, bad) {
        (Ok(g), Ok(b)) => (g, b),
        (g, b) => {
            return respond(&format!("good: {:?}\nbad: {:?}\n", g.err(), b.err()));
        }
    };

    let started = Instant::now();
    let mut last = None;
    while started.elapsed() < Duration::from_secs(6) {
        let now = format!("good {:?}, bad {:?}", good.is_healthy(), bad.is_healthy());
        if last.as_ref() != Some(&now) {
            report.push_str(&format!("{:>5} ms  {now}\n", started.elapsed().as_millis()));
            last = Some(now);
        }
        std::thread::sleep(Duration::from_millis(250));
    }

    for (label, b) in [("good", &good), ("bad", &bad)] {
        let request = http_req::Request::new().map_err(|_| ())?;
        request.set_uri("https://http-me.fastly.dev/anything").map_err(|_| ())?;
        let result = http_req::send_uncached(request, http_body::new().map_err(|_| ())?, b);
        let outcome = match result {
            Ok((response, _)) => format!("status {:?}", response.get_status()),
            Err(e) => format!("{:?}, detail {:?}", e.error, e.detail),
        };
        report.push_str(&format!("send through {label}: {outcome}\n"));
    }
    respond(&report)
}

struct HealthChecks;

impl http_incoming::Guest for HealthChecks {
    fn handle(request: http_incoming::Request, _body: http_body::Body) -> Result<(), ()> {
        let path = request.get_uri(1024).map_err(|_| ())?;
        let path = path.rsplit_once('/').map(|(_, p)| p).unwrap_or("");

        match path {
            "rules" => rules(),
            "health" => health(),
            _ => respond("try /rules or /health\n"),
        }
    }
}

bindings::export!(HealthChecks with_types_in bindings);
