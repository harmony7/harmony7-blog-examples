wit_bindgen::generate!({ path: "wit", world: "proxy-guest", generate_all });
use exports::wasi::http::incoming_handler::Guest;
use wasi::http::types::{Fields, IncomingRequest, OutgoingBody, OutgoingResponse, ResponseOutparam};

struct ProxyWorld;
impl Guest for ProxyWorld {
    fn handle(_request: IncomingRequest, response_out: ResponseOutparam) {
        let response = OutgoingResponse::new(Fields::new());
        let body = response.body().unwrap();
        ResponseOutparam::set(response_out, Ok(response));
        let stream = body.write().unwrap();
        stream.blocking_write_and_flush(b"hello from wasi:http/proxy\n").unwrap();
        drop(stream);
        OutgoingBody::finish(body, None).unwrap();
    }
}
export!(ProxyWorld);
