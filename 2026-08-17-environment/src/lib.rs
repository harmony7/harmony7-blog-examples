// <fold imports and boilerplate>
mod bindings;

use bindings::{exports::fastly::compute::http_incoming, fastly::compute::{http_body, http_resp}, wasi::cli::environment};

struct EnvironmentExample;
// </fold>

impl http_incoming::Guest for EnvironmentExample {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let mut lines = String::new();

        // <highlight>
        lines.push_str("FASTLY_* environment variables:\n");
        let mut fastly_vars: Vec<(String, String)> = environment::get_environment() // <1>
            .into_iter()
            .filter(|(k, _)| k.starts_with("FASTLY_"))
            .collect();
        fastly_vars.sort();
        for (k, v) in &fastly_vars {
            lines.push_str(&format!("  {k} = {v}\n"));
        }

        lines.push_str(&format!("\nget-arguments(): {:?}\n", environment::get_arguments())); // <2>
        lines.push_str(&format!("initial-cwd(): {:?}\n", environment::initial_cwd())); // <3>
        // </highlight>

        // <fold send the response>
        let response = http_resp::Response::new().map_err(|_| ())?;
        response
            .insert_header("content-type", b"text/plain")
            .map_err(|_| ())?;

        let out_body = http_body::new().map_err(|_| ())?;
        http_body::write(&out_body, lines.as_bytes()).map_err(|_| ())?;
        http_resp::send_downstream(response, out_body).map_err(|_| ())?;
        // </fold>

        Ok(())
    }
}

bindings::export!(EnvironmentExample with_types_in bindings);
