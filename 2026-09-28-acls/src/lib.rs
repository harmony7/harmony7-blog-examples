// <fold imports, and helpers for IP addresses, reading a body and sending a text response>
mod bindings;

use std::net::IpAddr;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{acl, http_body, http_downstream, http_resp, types},
};

fn to_wit(ip: IpAddr) -> types::IpAddress {
    match ip {
        IpAddr::V4(v4) => {
            let [a, b, c, d] = v4.octets();
            types::IpAddress::Ipv4((a, b, c, d))
        }
        IpAddr::V6(v6) => {
            let [a, b, c, d, e, f, g, h] = v6.segments();
            types::IpAddress::Ipv6((a, b, c, d, e, f, g, h))
        }
    }
}

fn from_wit(ip: &types::IpAddress) -> IpAddr {
    match *ip {
        types::IpAddress::Ipv4((a, b, c, d)) => IpAddr::from([a, b, c, d]),
        types::IpAddress::Ipv6((a, b, c, d, e, f, g, h)) => IpAddr::from([a, b, c, d, e, f, g, h]),
    }
}

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

fn respond(status: u16, text: String) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    response.set_status(status).map_err(|_| ())?;
    response.insert_header("content-type", b"text/plain").map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    http_resp::send_downstream(response, body).map_err(|_| ())
}
// </fold>

struct Acls;

impl http_incoming::Guest for Acls {
    fn handle(request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        // The client's address, unless an x-test-ip header asks about a different one.
        let ip = match request.get_header_value("x-test-ip", 64) {
            Ok(Some(v)) => std::str::from_utf8(&v).ok().and_then(|s| s.parse().ok()).map(to_wit),
            _ => http_downstream::downstream_client_ip_addr(&request),
        };
        let Some(ip) = ip else {
            return respond(400, "no IP address to look up\n".to_string());
        };

        let mut lines = format!("lookup {}\n", from_wit(&ip));

        if let Err(e) = acl::Acl::open("no-such-acl") {
            lines.push_str(&format!("open(\"no-such-acl\") = Err({e:?})\n"));
        }

        // <highlight>
        let blocklist = acl::Acl::open("blocklist").map_err(|_| ())?;
        let (status, verdict) = match blocklist.lookup(ip) {
        // </highlight>
            Ok(Some(body)) => {
                // <highlight>
                let json = String::from_utf8_lossy(&read_all(&body)).into_owned();
                // </highlight>
                let blocked = json.contains("\"BLOCK\"");
                (if blocked { 403 } else { 200 }, format!("match:\n{json}\n"))
            }
            Ok(None) => (200, "no match\n".to_string()),
            Err(e) => (500, format!("lookup: Err({e:?})\n")),
        };

        lines.push_str(&verdict);
        respond(status, lines)
    }
}

bindings::export!(Acls with_types_in bindings);
