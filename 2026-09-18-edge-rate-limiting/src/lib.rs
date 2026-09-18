// <fold imports, and helpers for formatting an IP and sending a text response>
mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{erl, http_body, http_downstream, http_resp, types},
};

fn format_ip(ip: Option<types::IpAddress>) -> String {
    match ip {
        Some(types::IpAddress::Ipv4((a, b, c, d))) => format!("{a}.{b}.{c}.{d}"),
        Some(types::IpAddress::Ipv6(p)) => {
            let parts = [p.0, p.1, p.2, p.3, p.4, p.5, p.6, p.7];
            parts.iter().map(|x| format!("{x:x}")).collect::<Vec<_>>().join(":")
        }
        None => "unknown".to_string(),
    }
}

fn respond(status: u16, text: String) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    response.set_status(status).map_err(|_| ())?;
    response.insert_header("content-type", b"text/plain").map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    http_resp::send_downstream(response, body).map_err(|_| ())
}
// </fold>

struct EdgeRateLimiting;

impl http_incoming::Guest for EdgeRateLimiting {
    fn handle(request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let entry = format_ip(http_downstream::downstream_client_ip_addr(&request));
        let mut lines = format!("{:<36}{entry:?}\n", "entry");

        // <highlight>
        let counter = erl::RateCounter::open("requests_by_ip").map_err(|_| ())?;
        let penalty_box = erl::PenaltyBox::open("banned_ips").map_err(|_| ())?;
        // </highlight>

        // Over 5 requests per second, averaged across 10 seconds, earns 60 seconds in the box.
        // <highlight>
        let limited = counter.check_rate(&entry, 1, 10, 5, &penalty_box, 60);
        // </highlight>
        lines.push_str(&format!("{:<36}{limited:?}\n", "check-rate (5 rps over 10s, 60s)"));

        lines.push_str(&format!("{:<36}{:?}\n", "lookup-rate (10s window)", counter.lookup_rate(&entry, 10)));
        lines.push_str(&format!("{:<36}{:?}\n", "lookup-count (last 60s)", counter.lookup_count(&entry, 60)));
        lines.push_str(&format!("{:<36}{:?}\n", "penalty-box has", penalty_box.has(&entry)));

        // Things the doc comments say shouldn't work.
        lines.push_str(&format!("{:<36}{:?}\n", "add, ttl 5s (under the 1m minimum)", penalty_box.add("someone-else", 5)));
        lines.push_str(&format!("{:<36}{:?}\n", "lookup-rate, window 7", counter.lookup_rate(&entry, 7)));
        let long_name = "x".repeat(2000);
        lines.push_str(&format!("{:<36}{}\n", "open, 2000-byte name", match erl::RateCounter::open(&long_name) {
            Ok(_) => "Ok".to_string(),
            Err(e) => format!("Err({e:?})"),
        }));

        if matches!(limited, Ok(true)) {
            return respond(429, lines);
        }
        respond(200, lines)
    }
}

bindings::export!(EdgeRateLimiting with_types_in bindings);
