// <fold imports, and helpers for reading a body and sending a text response>
mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_req, http_resp, shielding},
};

fn read_all(body: &http_body::Body) -> Vec<u8> {
    let mut out = Vec::new();
    while let Ok(chunk) = http_body::read(body, 4096) {
        if chunk.is_empty() {
            break;
        }
        out.extend(chunk);
    }
    out
}

fn respond(text: String) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    response.insert_header("content-type", b"text/plain").map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    http_resp::send_downstream(response, body).map_err(|_| ())
}
// </fold>

/// The layout the SDKs read out of `shield-info`'s string: one flag byte,
/// then the unencrypted and encrypted targets, each followed by a NUL.
fn parse_shield_info(info: &str) -> Option<(bool, &str, &str)> {
    let (flag, rest) = info.split_at_checked(1)?;
    let mut parts = rest.split('\0');
    let unencrypted = parts.next()?;
    let encrypted = parts.next()?;
    if parts.next() != Some("") || parts.next().is_some() {
        return None;
    }
    Some((flag != "\0", unencrypted, encrypted))
}

// <fold send_to: sends, then appends the status and the indented response body>
fn send_to(
    lines: &mut String,
    label: &str,
    request: http_req::Request,
    body: http_body::Body,
    to: &backend::Backend,
) {
    match http_req::send(request, body, to) {
        Ok((response, response_body)) => {
            let status = response.get_status().unwrap_or_default();
            lines.push_str(&format!("{label:<24}{status}\n"));
            // Indent what came back, so each hop's report nests inside the last.
            for line in String::from_utf8_lossy(&read_all(&response_body)).lines() {
                lines.push_str(&format!("    {line}\n"));
            }
        }
        Err(e) => lines.push_str(&format!("{label:<24}Err({:?})\n", e.error)),
    }
}
// </fold>

struct Shielding;

impl http_incoming::Guest for Shielding {
    fn handle(request: http_incoming::Request, request_body: http_body::Body) -> Result<(), ()> {
        let shield = match request.get_header_value("shield", 64) {
            Ok(Some(v)) => String::from_utf8_lossy(&v).into_owned(),
            _ => "pdx-or-us".to_string(),
        };
        let mut lines = String::new();

        // <highlight>
        let info = shielding::shield_info(&shield, 1024);
        // </highlight>
        lines.push_str(&format!("shield-info({shield:?}) = {info:?}\n"));

        let parsed = info.as_deref().ok().and_then(parse_shield_info);
        if let Some((running_on, _, _)) = parsed {
            lines.push_str(&format!("{:<24}{running_on}\n", "running on shield"));
        }

        if let Some((false, _unencrypted, encrypted)) = parsed {
            // Not on the shield: build a backend that points at it, and forward.
            let options = shielding::ShieldBackendOptions::new();
            options.set_first_byte_timeout(15_000);
            // <highlight>
            match shielding::backend_for_shield(encrypted, Some(&options)) {
            // </highlight>
                Ok(to_shield) => {
                    lines.push_str(&format!("{:<24}{:?}\n", "backend-for-shield", to_shield.get_name()));
                    send_to(&mut lines, "forwarded to shield", request, request_body, &to_shield);
                    return respond(lines);
                }
                Err(e) => lines.push_str(&format!("{:<24}Err({e:?})\n", "backend-for-shield")),
            }
        }

        // <highlight>
        // On the shield, or shielding isn't available: go to origin directly.
        // This is the last hop, so it's the only safe place to change the Host header.
        request.insert_header("host", b"http-me.fastly.dev").map_err(|_| ())?;
        // </highlight>
        let origin = backend::Backend::open("origin").map_err(|_| ())?;
        send_to(&mut lines, "sent to origin", request, request_body, &origin);
        respond(lines)
    }
}

bindings::export!(Shielding with_types_in bindings);
