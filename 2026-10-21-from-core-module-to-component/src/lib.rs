wit_bindgen::generate!({ path: "wit", world: "fastly-compute-guest", generate_all });

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
    fn handle(request: http_incoming::Request, _body: http_body::Body) -> Result<(), ()> {
        let uri = request.get_uri(8192).map_err(|_| ())?;
        respond(&format!("uri: {uri}\n"))
    }
}

export!(Service);
