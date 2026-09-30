mod bindings;

use std::time::Instant;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_req, http_resp},
};

/// One way for a backend request to go wrong, set up on purpose.
struct Case {
    name: &'static str,
    target: &'static str,
    uri: &'static str,
    use_tls: bool,
    tweak: fn(&backend::DynamicBackendOptions),
}

fn no_tweak(_: &backend::DynamicBackendOptions) {}

const CASES: &[Case] = &[
    // A request that works, for comparison.
    Case {
        name: "ok",
        target: "http-me.fastly.dev",
        uri: "/anything",
        use_tls: true,
        tweak: no_tweak,
    },
    // .invalid is reserved (RFC 2606), so this name can never resolve.
    Case {
        name: "dns",
        target: "no-such-host.invalid",
        uri: "/",
        use_tls: true,
        tweak: no_tweak,
    },
    // 192.0.2.1 is TEST-NET-1 (RFC 5737), reserved for documentation.
    Case {
        name: "reserved-ip",
        target: "192.0.2.1:80",
        uri: "/",
        use_tls: false,
        tweak: |o| o.connect_timeout(1_000),
    },
    // A real host, on a port that drops connection attempts unanswered.
    Case {
        name: "silent-port",
        target: "example.com:81",
        uri: "/",
        use_tls: false,
        tweak: |o| o.connect_timeout(1_000),
    },
    // badssl.com serves deliberately broken certificates. cert-hostname
    // turns on the hostname check, as its doc comment recommends.
    Case {
        name: "expired-cert",
        target: "expired.badssl.com",
        uri: "/",
        use_tls: true,
        tweak: |o| o.cert_hostname("expired.badssl.com"),
    },
    // Offer nothing newer than TLS 1.0 to a server that won't speak it.
    Case {
        name: "old-tls",
        target: "http-me.fastly.dev",
        uri: "/",
        use_tls: true,
        tweak: |o| o.tls_max_version(0x0301),
    },
    // http-me waits 3s before answering; we only give it 1s.
    Case {
        name: "slow-origin",
        target: "http-me.fastly.dev",
        uri: "/wait=3000",
        use_tls: true,
        tweak: |o| o.first_byte_timeout(1_000),
    },
];

fn describe(detail: &http_req::SendErrorDetail) -> String {
    use http_req::SendErrorDetail as D;
    match detail {
        D::DnsError(d) => format!("dns-error (rcode {:?}, info-code {:?})", d.rcode, d.info_code),
        D::TlsAlertReceived(d) => format!("tls-alert-received (id {:?})", d.id),
        D::H2Error(d) => format!(
            "h2-error (frame-type {}, error-code {})",
            d.frame_type, d.error_code
        ),
        D::Extra(_) => "extra (a case this guest's bindings predate)".to_string(),
        other => format!("{other:?}"),
    }
}

fn run(case: &Case) -> String {
    let options = backend::DynamicBackendOptions::new();
    options.use_tls(case.use_tls);
    (case.tweak)(&options);

    let backend = match backend::register_dynamic_backend(case.name, case.target, options) {
        Ok(b) => b,
        Err(e) => return format!("register-dynamic-backend failed: {e:?}"),
    };

    let request = http_req::Request::new().expect("request.new");
    request.set_method("GET").expect("set-method");
    request.set_uri(case.uri).expect("set-uri");
    let body = http_body::new().expect("body.new");

    let started = Instant::now();
    let result = http_req::send_uncached(request, body, &backend);
    let ms = started.elapsed().as_millis();

    match result {
        Ok((response, _body)) => {
            let status = response.get_status().expect("get-status");
            format!("ok, status {status} ({ms} ms)")
        }
        Err(http_req::ErrorWithDetail { detail, error }) => {
            let detail = match &detail {
                Some(d) => describe(d),
                None => "none".to_string(),
            };
            format!("error: {error:?}, detail: {detail} ({ms} ms)")
        }
    }
}

struct SendErrors;

impl http_incoming::Guest for SendErrors {
    fn handle(_request: http_incoming::Request, _body: http_body::Body) -> Result<(), ()> {
        let mut report = String::new();
        for case in CASES {
            report.push_str(&format!("{:<16} {}\n", case.name, run(case)));
        }

        let response = http_resp::Response::new().map_err(|_| ())?;
        response
            .insert_header("content-type", b"text/plain")
            .map_err(|_| ())?;
        let body = http_body::new().map_err(|_| ())?;
        http_body::write(&body, report.as_bytes()).map_err(|_| ())?;
        http_resp::send_downstream(response, body).map_err(|_| ())?;
        Ok(())
    }
}

bindings::export!(SendErrors with_types_in bindings);
