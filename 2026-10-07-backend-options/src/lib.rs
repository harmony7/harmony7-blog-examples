mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_resp},
};

fn respond(text: &str) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    http_resp::send_downstream(response, body).map_err(|_| ())
}

/// Reads back everything a backend will say about itself.
fn describe(b: &backend::Backend) -> String {
    let override_host = b
        .get_override_host(256)
        .map(|o| o.map(|v| String::from_utf8_lossy(&v).into_owned()));
    [
        format!("name:                  {}", b.get_name()),
        format!("is-dynamic:            {:?}", b.is_dynamic()),
        format!("host:                  {:?}", b.get_host(256)),
        format!("override-host:         {override_host:?}"),
        format!("port:                  {:?}", b.get_port()),
        format!("is-tls:                {:?}", b.is_tls()),
        format!("tls-min-version:       {:?}", b.get_tls_min_version()),
        format!("tls-max-version:       {:?}", b.get_tls_max_version()),
        format!("connect-timeout-ms:    {:?}", b.get_connect_timeout_ms()),
        format!("first-byte-timeout-ms: {:?}", b.get_first_byte_timeout_ms()),
        format!("between-bytes-ms:      {:?}", b.get_between_bytes_timeout_ms()),
        format!("http-keepalive-time:   {:?}", b.get_http_keepalive_time()),
        format!("tcp-keepalive-enable:  {:?}", b.get_tcp_keepalive_enable()),
        format!("tcp-keepalive-interval:{:?}", b.get_tcp_keepalive_interval()),
        format!("tcp-keepalive-probes:  {:?}", b.get_tcp_keepalive_probes()),
        format!("tcp-keepalive-time:    {:?}", b.get_tcp_keepalive_time()),
        format!("is-healthy:            {:?}", b.is_healthy()),
    ]
    .join("\n")
        + "\n"
}

/// A dynamic backend with every tuning option set to something other than
/// its default, so the getters have something to report.
fn tuned() -> Result<backend::Backend, ()> {
    let o = backend::DynamicBackendOptions::new();
    o.override_host("http-me.fastly.dev");
    o.connect_timeout(2_000);
    o.first_byte_timeout(5_000);
    o.between_bytes_timeout(3_000);
    o.tls_min_version(0x0303); // TLS 1.2
    o.tls_max_version(0x0304); // TLS 1.3
    o.cert_hostname("http-me.fastly.dev");
    o.sni_hostname("http-me.fastly.dev");
    o.http_keepalive_time_ms(30_000);
    o.tcp_keepalive_enable(1);
    o.tcp_keepalive_interval_secs(10);
    o.tcp_keepalive_probes(3);
    o.tcp_keepalive_time_secs(60);
    o.max_connections(50);
    o.max_use(100);
    o.max_lifetime_ms(60_000);
    o.prefer_ipv6(false);
    backend::register_dynamic_backend("tuned", "http-me.fastly.dev", o).map_err(|_| ())
}

struct BackendOptions;

impl http_incoming::Guest for BackendOptions {
    fn handle(request: http_incoming::Request, _body: http_body::Body) -> Result<(), ()> {
        let path = request.get_uri(1024).map_err(|_| ())?;
        let path = path.rsplit_once('/').map(|(_, p)| p).unwrap_or("");

        match path {
            "static" => respond(&describe(&backend::Backend::open("http_me").map_err(|_| ())?)),
            "dynamic" => respond(&describe(&tuned()?)),
            _ => respond("try /static or /dynamic\n"),
        }
    }
}

bindings::export!(BackendOptions with_types_in bindings);
