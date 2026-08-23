mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{http_body, http_downstream, http_resp},
};

// <fold call_growable, unchanged from the identity article>
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

struct HttpDownstreamTlsBotsExample;

impl http_incoming::Guest for HttpDownstreamTlsBotsExample {
    fn handle(request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let mut lines = String::new();

        // <highlight>
        lines.push_str("-- TLS --\n");
        let cipher = call_growable(|max_len| {
            http_downstream::downstream_tls_cipher_openssl_name(&request, max_len)
        });
        lines.push_str(&format!("cipher-openssl-name: {cipher:?}\n"));
        // <fold protocol, client-hello, raw-client-certificate — same growable byte-list shape>
        let protocol =
            call_growable(|max_len| http_downstream::downstream_tls_protocol(&request, max_len));
        lines.push_str(&format!("protocol: {protocol:?}\n"));
        let client_hello = call_growable(|max_len| {
            http_downstream::downstream_tls_client_hello(&request, max_len)
        });
        lines.push_str(&format!("client-hello: {client_hello:?}\n"));
        let raw_cert = call_growable(|max_len| {
            http_downstream::downstream_tls_raw_client_certificate(&request, max_len)
        });
        lines.push_str(&format!("raw-client-certificate: {raw_cert:?}\n"));
        // </fold>
        let verify_result = http_downstream::downstream_tls_client_cert_verify_result(&request);
        lines.push_str(&format!("client-cert-verify-result: {verify_result:?}\n"));
        let servername = call_growable(|max_len| {
            http_downstream::downstream_tls_client_servername(&request, max_len)
        });
        lines.push_str(&format!("client-servername: {servername:?}\n"));
        let ja3 = http_downstream::downstream_tls_ja3_md5(&request);
        lines.push_str(&format!("ja3-md5: {ja3:?}\n"));
        let ja4 = call_growable(|max_len| http_downstream::downstream_tls_ja4(&request, max_len));
        lines.push_str(&format!("ja4: {ja4:?}\n"));

        lines.push_str("\n-- Bot detection --\n");
        let analyzed = http_downstream::downstream_bot_analyzed(&request);
        lines.push_str(&format!("bot-analyzed: {analyzed:?}\n"));
        let detected = http_downstream::downstream_bot_detected(&request);
        lines.push_str(&format!("bot-detected: {detected:?}\n"));
        // <fold bot-name, bot-category, bot-category-kind, bot-verified — same shapes as above>
        let name = call_growable(|max_len| http_downstream::downstream_bot_name(&request, max_len));
        lines.push_str(&format!("bot-name: {name:?}\n"));
        let category =
            call_growable(|max_len| http_downstream::downstream_bot_category(&request, max_len));
        lines.push_str(&format!("bot-category: {category:?}\n"));
        let category_kind = http_downstream::downstream_bot_category_kind(&request);
        lines.push_str(&format!("bot-category-kind: {category_kind:?}\n"));
        let verified = http_downstream::downstream_bot_verified(&request);
        lines.push_str(&format!("bot-verified: {verified:?}\n"));
        // </fold>

        lines.push_str("\n-- VPN / proxy detection --\n");
        let checks: [(
            &str,
            fn(&http_incoming::Request) -> Result<Option<bool>, http_downstream::Error>,
        ); 10] = [
            ("is-anonymous", http_downstream::downstream_resvpnproxy_is_anonymous),
            ("is-anonymous-vpn", http_downstream::downstream_resvpnproxy_is_anonymous_vpn),
            ("is-hosting-provider", http_downstream::downstream_resvpnproxy_is_hosting_provider),
            ("is-proxy-over-vpn", http_downstream::downstream_resvpnproxy_is_proxy_over_vpn),
            ("is-public-proxy", http_downstream::downstream_resvpnproxy_is_public_proxy),
            ("is-relay-proxy", http_downstream::downstream_resvpnproxy_is_relay_proxy),
            ("is-residential-proxy", http_downstream::downstream_resvpnproxy_is_residential_proxy),
            ("is-smart-dns-proxy", http_downstream::downstream_resvpnproxy_is_smart_dns_proxy),
            ("is-tor-exit-node", http_downstream::downstream_resvpnproxy_is_tor_exit_node),
            ("is-vpn-datacenter", http_downstream::downstream_resvpnproxy_is_vpn_datacenter),
        ];
        for (name, check) in checks {
            let result = check(&request);
            lines.push_str(&format!("resvpnproxy-{name}: {result:?}\n"));
        }
        let vpn_service_name = call_growable(|max_len| {
            http_downstream::downstream_resvpnproxy_vpn_service_name(&request, max_len)
        });
        lines.push_str(&format!("resvpnproxy-vpn-service-name: {vpn_service_name:?}\n"));
        // </highlight>

        // <fold send the response, unchanged>
        let response = http_resp::Response::new().map_err(|_| ())?;
        response.insert_header("content-type", b"text/plain").map_err(|_| ())?;
        let out_body = http_body::new().map_err(|_| ())?;
        http_body::write(&out_body, lines.as_bytes()).map_err(|_| ())?;
        http_resp::send_downstream(response, out_body).map_err(|_| ())?;
        // </fold>

        Ok(())
    }
}

bindings::export!(HttpDownstreamTlsBotsExample with_types_in bindings);
