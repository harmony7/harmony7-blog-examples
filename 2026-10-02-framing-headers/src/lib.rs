mod bindings;

use std::net::IpAddr;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_req, http_resp, http_types, types},
};

use http_types::{ContentEncodings, FramingHeadersMode};

fn text_body(text: &str) -> Result<http_body::Body, ()> {
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    Ok(body)
}

/// A six-byte body, sent with the framing mode and headers given.
fn framed(mode: FramingHeadersMode, headers: &[(&str, &str)]) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    for (name, value) in headers {
        response.append_header(name, value.as_bytes()).map_err(|_| ())?;
    }
    response.set_framing_headers_mode(mode).map_err(|_| ())?;
    http_resp::send_downstream(response, text_body("hello\n")?).map_err(|_| ())
}

/// Starts the response, then writes the body in three pieces after it's
/// already on its way.
fn streamed() -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_resp::send_downstream_streaming(response, &body).map_err(|_| ())?;
    for piece in ["one\n", "two\n", "three\n"] {
        http_body::write(&body, piece.as_bytes()).map_err(|_| ())?;
    }
    http_body::close(body).map_err(|_| ())
}

fn read_all(body: &http_body::Body) -> Vec<u8> {
    let mut out = Vec::new();
    while let Ok(chunk) = http_body::read(body, 8192) {
        if chunk.is_empty() {
            break;
        }
        out.extend(chunk);
    }
    out
}

fn to_std(ip: types::IpAddress) -> IpAddr {
    match ip {
        types::IpAddress::Ipv4((a, b, c, d)) => IpAddr::from([a, b, c, d]),
        types::IpAddress::Ipv6((a, b, c, d, e, f, g, h)) => IpAddr::from([a, b, c, d, e, f, g, h]),
    }
}

fn header(response: &http_resp::Response, name: &str) -> String {
    match response.get_header_value(name, 256) {
        Ok(Some(v)) => String::from_utf8_lossy(&v).into_owned(),
        _ => "(none)".to_string(),
    }
}

/// Fetches http-me's gzipped JSON, with and without automatic decompression.
fn gzip() -> Result<(), ()> {
    let backend = backend::Backend::open("http_me").map_err(|_| ())?;
    let mut report = String::new();

    for decompress in [false, true] {
        let request = http_req::Request::new().map_err(|_| ())?;
        request.set_uri("https://http-me.fastly.dev/gzip").map_err(|_| ())?;
        request.insert_header("accept-encoding", b"gzip").map_err(|_| ())?;
        if decompress {
            request
                .set_auto_decompress_response(ContentEncodings::GZIP)
                .map_err(|_| ())?;
        }

        let (response, body) =
            http_req::send_uncached(request, http_body::new().map_err(|_| ())?, &backend)
                .map_err(|_| ())?;
        let bytes = read_all(&body);

        report.push_str(&format!(
            "auto-decompress {decompress}:\n  content-encoding: {}\n  content-length: {}\n  \
             {} bytes, starting {:02x?}\n",
            header(&response, "content-encoding"),
            header(&response, "content-length"),
            bytes.len(),
            &bytes[..bytes.len().min(4)],
        ));
        report.push_str(&format!(
            "  remote address: {:?}, port {:?}\n",
            response.get_remote_ip_addr().map(to_std),
            response.get_remote_port(),
        ));
    }

    let response = http_resp::Response::new().map_err(|_| ())?;
    http_resp::send_downstream(response, text_body(&report)?).map_err(|_| ())
}

/// Asks http-me to echo the request it received, with automatic
/// decompression on and no Accept-Encoding of our own.
fn accept_encoding() -> Result<(), ()> {
    let backend = backend::Backend::open("http_me").map_err(|_| ())?;
    let request = http_req::Request::new().map_err(|_| ())?;
    request.set_uri("https://http-me.fastly.dev/anything").map_err(|_| ())?;
    request
        .set_auto_decompress_response(ContentEncodings::GZIP)
        .map_err(|_| ())?;
    let (_response, body) =
        http_req::send_uncached(request, http_body::new().map_err(|_| ())?, &backend)
            .map_err(|_| ())?;
    let echoed = read_all(&body);

    let response = http_resp::Response::new().map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, &echoed).map_err(|_| ())?;
    http_resp::send_downstream(response, body).map_err(|_| ())
}

fn no_keepalive() -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    let result = response.set_http_keepalive_mode(http_resp::KeepaliveMode::NoKeepalive);
    let text = format!("set-http-keepalive-mode(no-keepalive) = {result:?}\n");
    http_resp::send_downstream(response, text_body(&text)?).map_err(|_| ())
}

struct FramingHeaders;

impl http_incoming::Guest for FramingHeaders {
    fn handle(request: http_incoming::Request, _body: http_body::Body) -> Result<(), ()> {
        let path = request.get_uri(1024).map_err(|_| ())?;
        let path = path.rsplit_once('/').map(|(_, p)| p).unwrap_or("");

        use FramingHeadersMode::{Automatic, ManuallyFromHeaders};
        match path {
            // A wrong Content-Length, which automatic mode throws away.
            "auto" => framed(Automatic, &[("content-length", "999")]),
            "manual" => framed(ManuallyFromHeaders, &[("content-length", "6")]),
            "manual-short" => framed(ManuallyFromHeaders, &[("content-length", "3")]),
            "manual-long" => framed(ManuallyFromHeaders, &[("content-length", "50")]),
            // Both framing headers at once, which HTTP/1.1 doesn't permit.
            "manual-invalid" => framed(
                ManuallyFromHeaders,
                &[("content-length", "6"), ("transfer-encoding", "chunked")],
            ),
            "stream" => streamed(),
            "gzip" => gzip(),
            "accept-encoding" => accept_encoding(),
            "no-keepalive" => no_keepalive(),
            _ => {
                let response = http_resp::Response::new().map_err(|_| ())?;
                response.set_status(404).map_err(|_| ())?;
                http_resp::send_downstream(response, text_body("not found\n")?).map_err(|_| ())
            }
        }
    }
}

bindings::export!(FramingHeaders with_types_in bindings);
