wit_bindgen::generate!({ path: "../wit", world: "service", generate_all });

use exports::fastly::compute::http_incoming;
use fastly::compute::{http_body, http_resp};

fn respond(text: &str) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    http_resp::send_downstream(response, body).map_err(|_| ())
}

struct Service;

impl http_incoming::Guest for Service {
    fn handle(_request: http_incoming::Request, _body: http_body::Body) -> Result<(), ()> {
        let mut report = String::new();

        // Imports wasi:filesystem.
        #[cfg(feature = "fs")]
        report.push_str(&format!("fs: {:?}\n", std::fs::read_to_string("/etc/hostname")));

        // Imports wasi:sockets, and wasi:filesystem along with it.
        #[cfg(feature = "net")]
        report.push_str(&format!(
            "net: {:?}\n",
            std::net::TcpStream::connect("192.0.2.1:80").map(|_| ())
        ));

        report.push_str("handled\n");
        respond(&report)
    }
}

export!(Service);
