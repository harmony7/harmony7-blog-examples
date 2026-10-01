mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{http_body, http_resp, types},
};

/// Collects every header name by paging through a names function with a
/// deliberately small buffer, noting each page in `log`.
fn all_names(
    mut page: impl FnMut(u64, u32) -> Result<(String, Option<u32>), types::Error>,
    mut max_len: u64,
    log: &mut String,
) -> Result<Vec<String>, types::Error> {
    let mut names = Vec::new();
    let mut cursor = 0;
    loop {
        match page(max_len, cursor) {
            Ok((buf, next)) => {
                // Each name is followed by a NUL, so the last split is empty.
                let got: Vec<&str> = buf.split('\0').filter(|n| !n.is_empty()).collect();
                log.push_str(&format!("  cursor {cursor}: {got:?}, next {next:?}\n"));
                names.extend(got.into_iter().map(String::from));
                match next {
                    Some(n) => cursor = n,
                    None => return Ok(names),
                }
            }
            // Not even one name fit, so take the size the host suggests.
            Err(types::Error::BufferLen(needed)) => max_len = needed,
            Err(e) => return Err(e),
        }
    }
}

/// Reads every value of one response header, split on the NUL terminators.
fn values(response: &http_resp::Response, name: &str) -> Vec<String> {
    match response.get_header_values(name, 1024, 0) {
        Ok((buf, _)) => buf
            .split(|b| *b == 0)
            .filter(|v| !v.is_empty())
            .map(|v| String::from_utf8_lossy(v).into_owned())
            .collect(),
        Err(_) => Vec::new(),
    }
}

struct Headers;

impl http_incoming::Guest for Headers {
    fn handle(request: http_incoming::Request, _body: http_body::Body) -> Result<(), ()> {
        let mut report = String::new();

        // Every header name on the incoming request, 24 bytes at a time.
        report.push_str("request header names:\n");
        let names = all_names(|len, cur| request.get_header_names(len, cur), 24, &mut report)
            .map_err(|_| ())?;
        report.push_str(&format!("  all: {names:?}\n"));

        let version = request.get_version().map_err(|_| ())?;
        report.push_str(&format!("request version: {version:?}\n\n"));

        let response = http_resp::Response::new().map_err(|_| ())?;

        // insert-header replaces; append-header adds another instance.
        response.insert_header("content-type", b"text/plain").map_err(|_| ())?;
        response.append_header("set-cookie", b"theme=dark").map_err(|_| ())?;
        response.append_header("set-cookie", b"lang=en").map_err(|_| ())?;

        // set-header-values takes every value at once, each one terminated
        // by a NUL, and replaces whatever was there.
        response
            .set_header_values("x-color", b"red\0green\0blue\0")
            .map_err(|_| ())?;
        // The same call without the final NUL.
        response
            .set_header_values("x-shape", b"circle\0square")
            .map_err(|_| ())?;

        // Removing a header that exists, and one that never did.
        response.insert_header("x-temporary", b"1").map_err(|_| ())?;
        let removed = response.remove_header("x-temporary");
        let not_there = response.remove_header("x-never-set");
        report.push_str(&format!("remove-header(\"x-temporary\") = {removed:?}\n"));
        report.push_str(&format!("remove-header(\"x-never-set\") = {not_there:?}\n\n"));

        response.set_status(207).map_err(|_| ())?;
        let status = response.get_status().map_err(|_| ())?;
        report.push_str(&format!("response status: {status}\n"));

        report.push_str("response headers, read back:\n");
        let mut scratch = String::new();
        let names = all_names(|len, cur| response.get_header_names(len, cur), 256, &mut scratch)
            .map_err(|_| ())?;
        for name in names {
            report.push_str(&format!("  {name}: {:?}\n", values(&response, &name)));
        }

        let body = http_body::new().map_err(|_| ())?;
        http_body::write(&body, report.as_bytes()).map_err(|_| ())?;
        http_resp::send_downstream(response, body).map_err(|_| ())?;
        Ok(())
    }
}

bindings::export!(Headers with_types_in bindings);
