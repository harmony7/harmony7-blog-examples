mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{geo, http_body, http_resp, types},
};

struct GeoExample;

impl http_incoming::Guest for GeoExample {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        // <highlight>
        let ip_addr = types::IpAddress::Ipv4((8, 8, 8, 8));

        let mut max_len: u64 = 1024;
        let result = loop {
            match geo::lookup(ip_addr.clone(), max_len) {
                Ok(json) => break Ok(json),
                Err(geo::Error::BufferLen(needed)) => max_len = needed,
                Err(e) => break Err(e),
            }
        };
        // </highlight>

        let msg = match result {
            Ok(json) => format!("geo.lookup(8.8.8.8):\n\n{json}\n"),
            Err(e) => format!("geo.lookup(8.8.8.8) failed:\n\n{e:?}\n"),
        };

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

bindings::export!(GeoExample with_types_in bindings);
