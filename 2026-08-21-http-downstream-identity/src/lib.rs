mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{http_body, http_downstream, http_resp, types},
};

// <fold call_growable, generic buffer-retry helper>
fn call_growable<T>(
    mut f: impl FnMut(u64) -> Result<T, http_downstream::Error>,
) -> Result<T, http_downstream::Error> {
    let mut max_len: u64 = 256;
    loop {
        match f(max_len) {
            Ok(v) => return Ok(v),
            Err(http_downstream::Error::BufferLen(needed)) => max_len = needed,
            Err(e) => return Err(e),
        }
    }
}
// </fold>

// <fold header_names, cursor pagination, same shape as get_header_values>
fn header_names(request: &http_incoming::Request) -> Result<Vec<String>, http_downstream::Error> {
    let mut names = Vec::new();
    let mut cursor: u32 = 0;
    let mut max_len: u64 = 256;

    loop {
        match http_downstream::downstream_original_header_names(request, max_len, cursor) {
            Ok((text, more)) => {
                names.extend(
                    text.split('\0')
                        .filter(|chunk| !chunk.is_empty())
                        .map(|chunk| chunk.to_string()),
                );
                match more {
                    Some(next_cursor) => cursor = next_cursor,
                    None => return Ok(names),
                }
            }
            Err(http_downstream::Error::BufferLen(needed)) => max_len = needed,
            Err(e) => return Err(e),
        }
    }
}
// </fold>

// <fold format_ip>
fn format_ip(ip: Option<types::IpAddress>) -> String {
    match ip {
        Some(types::IpAddress::Ipv4((a, b, c, d))) => format!("{a}.{b}.{c}.{d}"),
        Some(types::IpAddress::Ipv6(parts)) => format!("{parts:?}"),
        None => "(none)".to_string(),
    }
}
// </fold>

struct HttpDownstreamIdentityExample;

impl http_incoming::Guest for HttpDownstreamIdentityExample {
    fn handle(request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        // <highlight>
        let names = header_names(&request).map_err(|_| ())?;
        let count = http_downstream::downstream_original_header_count(&request).map_err(|_| ())?;

        let client_ip = format_ip(http_downstream::downstream_client_ip_addr(&request));
        let server_ip = format_ip(http_downstream::downstream_server_ip_addr(&request));

        let h2_fingerprint = call_growable(|max_len| {
            http_downstream::downstream_client_h2_fingerprint(&request, max_len)
        });
        let request_id = call_growable(|max_len| {
            http_downstream::downstream_client_request_id(&request, max_len)
        });
        let oh_fingerprint = call_growable(|max_len| {
            http_downstream::downstream_client_oh_fingerprint(&request, max_len)
        });
        let ddos_detected = http_downstream::downstream_client_ddos_detected(&request);
        let compliance_region = call_growable(|max_len| {
            http_downstream::downstream_compliance_region(&request, max_len)
        });
        let fastly_key_valid = http_downstream::fastly_key_is_valid(&request);
        let visits_service = http_downstream::downstream_visits_this_service();
        let visits_pop = http_downstream::downstream_visits_this_pop();
        // </highlight>

        let msg = format!(
            "downstream-original-header-count: {count}\n\
             downstream-original-header-names: {names:?}\n\
             downstream-client-ip-addr:  {client_ip}\n\
             downstream-server-ip-addr:  {server_ip}\n\
             downstream-client-h2-fingerprint: {h2_fingerprint:?}\n\
             downstream-client-request-id:     {request_id:?}\n\
             downstream-client-oh-fingerprint: {oh_fingerprint:?}\n\
             downstream-client-ddos-detected:  {ddos_detected:?}\n\
             downstream-compliance-region:     {compliance_region:?}\n\
             fastly-key-is-valid:        {fastly_key_valid:?}\n\
             downstream-visits-this-service: {visits_service:?}\n\
             downstream-visits-this-pop:     {visits_pop:?}\n"
        );

        // <fold send the response, unchanged>
        let response = http_resp::Response::new().map_err(|_| ())?;
        response.insert_header("content-type", b"text/plain").map_err(|_| ())?;
        let out_body = http_body::new().map_err(|_| ())?;
        http_body::write(&out_body, msg.as_bytes()).map_err(|_| ())?;
        http_resp::send_downstream(response, out_body).map_err(|_| ())?;
        // </fold>

        Ok(())
    }
}

bindings::export!(HttpDownstreamIdentityExample with_types_in bindings);
