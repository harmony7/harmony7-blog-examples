mod bindings;

use std::time::Instant;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{async_io, backend, http_body, http_req, http_resp},
    wasi::{clocks::monotonic_clock, io::poll},
};

fn respond(text: &str) -> Result<(), ()> {
    let response = http_resp::Response::new().map_err(|_| ())?;
    let body = http_body::new().map_err(|_| ())?;
    http_body::write(&body, text.as_bytes()).map_err(|_| ())?;
    http_resp::send_downstream(response, body).map_err(|_| ())
}

fn start(backend: &backend::Backend, uri: &str) -> Result<http_req::PendingResponse, ()> {
    let request = http_req::Request::new().map_err(|_| ())?;
    request.set_uri(uri).map_err(|_| ())?;
    http_req::send_async_uncached(request, http_body::new().map_err(|_| ())?, backend).map_err(|_| ())
}

/// Waits on a timer through wasi:io.
fn timer() -> Result<(), ()> {
    let started = Instant::now();
    let short = monotonic_clock::subscribe_duration(200_000_000); // 200 ms
    let long = monotonic_clock::subscribe_duration(500_000_000); // 500 ms
    let ready = poll::poll(&[&long, &short]);
    let first = started.elapsed().as_millis();
    long.block();
    let second = started.elapsed().as_millis();
    respond(&format!(
        "poll returned {ready:?} after {first} ms\nblock on the long timer returned after {second} ms\n"
    ))
}

/// Waits on a backend through async-io, with a timeout.
fn backend_wait() -> Result<(), ()> {
    let http_me = backend::Backend::open("http_me").map_err(|_| ())?;
    let started = Instant::now();
    let slow = start(&http_me, "https://http-me.fastly.dev/wait=1000")?;
    let result = async_io::select_with_timeout(&[&slow], 300);
    let first = started.elapsed().as_millis();
    let result2 = async_io::select_with_timeout(&[&slow], 5_000);
    let second = started.elapsed().as_millis();
    respond(&format!(
        "select-with-timeout(300) returned {result:?} after {first} ms\nselect-with-timeout(5000) returned {result2:?} after {second} ms\n"
    ))
}

#[cfg(feature = "mix")]
fn mixed() -> Result<(), ()> {
    let http_me = backend::Backend::open("http_me").map_err(|_| ())?;
    let slow = start(&http_me, "https://http-me.fastly.dev/wait=1000")?;
    let timer = monotonic_clock::subscribe_duration(300_000_000);
    let _ = async_io::select(&[&slow, &timer]);
    Ok(())
}

struct TwoPollables;

impl http_incoming::Guest for TwoPollables {
    fn handle(request: http_incoming::Request, _body: http_body::Body) -> Result<(), ()> {
        let path = request.get_uri(1024).map_err(|_| ())?;
        let path = path.rsplit_once('/').map(|(_, p)| p).unwrap_or("");
        match path {
            "timer" => timer(),
            "backend" => backend_wait(),
            #[cfg(feature = "mix")]
            "mixed" => mixed(),
            _ => respond("try /timer or /backend\n"),
        }
    }
}

bindings::export!(TwoPollables with_types_in bindings);
