mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_req, http_resp},
};

fn text_body(text: &str) -> Result<http_body::Body, ()> {
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    Ok(body)
}

/// Proxies http-me's echo of this request, with a line added to the front
/// of a body this program never reads.
fn front() -> Result<(), ()> {
    let backend = backend::Backend::open("http_me").map_err(|_| ())?;
    let request = http_req::Request::new().map_err(|_| ())?;
    request.set_uri("https://http-me.fastly.dev/anything").map_err(|_| ())?;
    let (response, body) =
        http_req::send_uncached(request, http_body::new().map_err(|_| ())?, &backend)
            .map_err(|_| ())?;

    http_body::write_front(&body, b"// fetched through Compute\n").map_err(|_| ())?;

    // The backend's Content-Length is now wrong, but automatic framing
    // discards it and computes a new one.
    http_resp::send_downstream(response, body).map_err(|_| ())
}

/// Tries write-front on a body that's already streaming to the client.
fn front_streaming() -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_resp::send_downstream_streaming(response, &body).map_err(|_| ())?;
    http_body::write(&body, b"already sent\n").map_err(|_| ())?;

    let result = http_body::write_front(&body, b"too late\n");
    let text = format!("write-front after streaming started = {result:?}\n");
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    http_body::close(body).map_err(|_| ())
}

/// Streams a body, then adds a trailer that depends on everything written.
fn trailers() -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    // Announce the trailer up front, as HTTP recommends.
    response.insert_header("trailer", b"x-body-length").map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_resp::send_downstream_streaming(response, &body).map_err(|_| ())?;

    let mut length = 0;
    for piece in ["one\n", "two\n", "three\n"] {
        length += http_body::write(&body, piece.as_bytes()).map_err(|_| ())?;
    }
    http_body::append_trailer(&body, "x-body-length", length.to_string().as_bytes())
        .map_err(|_| ())?;
    http_body::close(body).map_err(|_| ())
}

fn trailer_names(body: &http_body::Body) -> String {
    match http_body::get_trailer_names(body, 1024, 0) {
        Ok((names, _)) => format!("{:?}", names.split('\0').filter(|n| !n.is_empty()).collect::<Vec<_>>()),
        Err(e) => format!("{e:?}"),
    }
}

/// Reads the request's trailers before and after reading its body.
fn echo_trailers(request_body: http_body::Body) -> Result<(), ()> {
    let mut report = format!("before reading: {}\n", trailer_names(&request_body));

    let mut bytes = 0;
    loop {
        let chunk = http_body::read(&request_body, 8192).map_err(|_| ())?;
        if chunk.is_empty() {
            break;
        }
        bytes += chunk.len();
    }
    report.push_str(&format!("read {bytes} body bytes\n"));
    report.push_str(&format!("after reading: {}\n", trailer_names(&request_body)));

    if let Ok(Some(v)) = http_body::get_trailer_value(&request_body, "x-checksum", 256) {
        report.push_str(&format!("x-checksum: {}\n", String::from_utf8_lossy(&v)));
    }

    let response = http_resp::Response::new().map_err(|_| ())?;
    http_resp::send_downstream(response, text_body(&report)?).map_err(|_| ())
}

struct WriteFrontAndTrailers;

impl http_incoming::Guest for WriteFrontAndTrailers {
    fn handle(request: http_incoming::Request, body: http_body::Body) -> Result<(), ()> {
        let path = request.get_uri(1024).map_err(|_| ())?;
        let path = path.rsplit_once('/').map(|(_, p)| p).unwrap_or("");

        match path {
            "front" => front(),
            "front-streaming" => front_streaming(),
            "trailers" => trailers(),
            "echo-trailers" => echo_trailers(body),
            _ => {
                let response = http_resp::Response::new().map_err(|_| ())?;
                response.set_status(404).map_err(|_| ())?;
                http_resp::send_downstream(response, text_body("not found\n")?).map_err(|_| ())
            }
        }
    }
}

bindings::export!(WriteFrontAndTrailers with_types_in bindings);
