// <fold imports and the response helper>
mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{http_body, http_resp, log},
};

fn send(lines: String) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    response.insert_header("content-type", b"text/plain").map_err(|_| ())?;
    let out_body = http_body::new().map_err(|_| ())?;
    http_body::write(&out_body, lines.as_bytes()).map_err(|_| ())?;
    http_resp::send_downstream(response, out_body).map_err(|_| ())?;
    Ok(())
}
// </fold>

/// Reports what `open` did with a given name, and writes to the endpoint if
/// it produced one. `write` has no return value to report.
fn probe(lines: &mut String, label: &str, name: &str) {
    match log::Endpoint::open(name) {
        Ok(endpoint) => {
            // <highlight>
            endpoint.write(format!("hello from {label}").as_bytes());
            // </highlight>
            lines.push_str(&format!("{label:<22} open: Ok, wrote 1 event\n"));
        }
        Err(e) => lines.push_str(&format!("{label:<22} open: Err({e:?})\n")),
    }
}

struct RealtimeLogging;

impl http_incoming::Guest for RealtimeLogging {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let mut lines = String::new();

        // <highlight>
        // A name you'd configure as a logging endpoint on a real service.
        // Locally, nothing configures it.
        probe(&mut lines, "\"my_endpoint\"", "my_endpoint");

        // A name that is not configured anywhere. The doc comment says this
        // still returns a usable endpoint, and that writes to it succeed.
        probe(&mut lines, "\"not_configured\"", "not_configured");

        // The three conditions the doc comment puts on a name.
        probe(&mut lines, "\"\" (empty)", "");
        probe(&mut lines, "\"has:colon\"", "has:colon");
        probe(&mut lines, "\"has\\nnewline\"", "has\nnewline");

        // Reserved for debugging, per the same doc comment.
        probe(&mut lines, "\"stdout\"", "stdout");
        probe(&mut lines, "\"stderr\"", "stderr");
        // </highlight>

        // For contrast: the WASI path println! takes.
        println!("this line went to wasi:cli/stdout, not to a log endpoint");

        send(lines)
    }
}

bindings::export!(RealtimeLogging with_types_in bindings);
