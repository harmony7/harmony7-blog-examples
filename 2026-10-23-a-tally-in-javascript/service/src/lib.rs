wit_bindgen::generate!({ path: "../wit", world: "service", generate_all });

use exports::fastly::compute::http_incoming;
use fastly::compute::{http_body, http_resp};
use local::main::tally::Tally;

fn respond(text: &str) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    http_resp::send_downstream(response, body).map_err(|_| ())
}

struct Service;

impl http_incoming::Guest for Service {
    fn handle(_request: http_incoming::Request, _body: http_body::Body) -> Result<(), ()> {
        let requests = Tally::new("requests");
        let bytes = Tally::new("bytes");
        requests.add(1);
        bytes.add(40);
        let total = bytes.add(2);
        drop(bytes);
        eprintln!("service: dropped its handle to bytes");
        respond(&format!("bytes reached {total}\n"))
    }
}

export!(Service);
