// <fold imports, and a helper for sending a text response>
mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_resp, security},
};
use serde::Deserialize;

fn respond(status: u16, text: String) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    response.set_status(status).map_err(|_| ())?;
    response.insert_header("content-type", b"text/plain").map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    http_resp::send_downstream(response, body).map_err(|_| ())
}
// </fold>

/// The verdict document, as the SDKs read it.
#[derive(Deserialize, Debug)]
#[allow(dead_code)] // Some fields are only printed.
struct Verdict {
    #[serde(alias = "response")]
    waf_response: i32,
    redirect_url: String,
    tags: Vec<String>,
    verdict: String,
    decision_ms: u64,
}

fn options(corp: &str, workspace: &str) -> security::InspectOptions<'static> {
    security::InspectOptions {
        corp: Some(corp.to_string()),
        workspace: Some(workspace.to_string()),
        override_client_ip: None,
        extra: None,
    }
}

struct NextGenWaf;

impl http_incoming::Guest for NextGenWaf {
    fn handle(request: http_incoming::Request, request_body: http_body::Body) -> Result<(), ()> {
        let mut lines = String::new();

        // Two calls that should fail: no workspace, and too small a buffer.
        let mut no_workspace = options("my-corp", "my-workspace-id");
        no_workspace.workspace = None;
        let r = security::inspect(&request, &request_body, &no_workspace, 16 * 1024);
        lines.push_str(&format!("no workspace:  {r:?}\n"));
        let r = security::inspect(&request, &request_body, &options("my-corp", "my-workspace-id"), 16);
        lines.push_str(&format!("max-len 16:    {r:?}\n"));

        // <highlight>
        let json = match security::inspect(&request, &request_body, &options("my-corp", "my-workspace-id"), 16 * 1024) {
        // </highlight>
            Ok(json) => json,
            Err(e) => return respond(500, format!("{lines}inspect: Err({e:?})\n")),
        };
        lines.push_str(&format!("inspect:       {json}\n"));

        let verdict: Verdict = match serde_json::from_str(&json) {
            Ok(v) => v,
            Err(e) => return respond(500, format!("{lines}unreadable verdict: {e}\n")),
        };
        lines.push_str(&format!("parsed:        {verdict:?}\n"));

        if verdict.verdict != "allow" {
            let status = u16::try_from(verdict.waf_response).unwrap_or(403);
            return respond(status, lines);
        }

        // Inspection only borrowed the request and body, so they're still ours to forward.
        // <highlight>
        let origin = backend::Backend::open("origin").map_err(|_| ())?;
        request.set_uri("https://http-me.fastly.dev/body=allowed").map_err(|_| ())?;
        request.insert_header("host", b"http-me.fastly.dev").map_err(|_| ())?;
        let (_response, body) = bindings::fastly::compute::http_req::send(request, request_body, &origin).map_err(|_| ())?;
        // </highlight>
        let mut from_origin = Vec::new();
        while let Ok(chunk) = http_body::read(&body, 4096) {
            if chunk.is_empty() {
                break;
            }
            from_origin.extend(chunk);
        }
        lines.push_str(&format!("origin said:   {}\n", String::from_utf8_lossy(&from_origin)));
        respond(200, lines)
    }
}

bindings::export!(NextGenWaf with_types_in bindings);
