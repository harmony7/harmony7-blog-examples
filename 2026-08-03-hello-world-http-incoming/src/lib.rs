mod bindings;

use bindings::{exports::fastly::compute::http_incoming, fastly::compute::http_body};

struct HelloWorld;

impl http_incoming::Guest for HelloWorld {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        println!("Hello, world!");
        Ok(())
    }
}

bindings::export!(HelloWorld with_types_in bindings);
