# Write-front and trailers: both ends of a body

Adds a line to the front of a backend's body without reading it (`/front`), tries the same on a body that's already streaming (`/front-streaming`), streams a response that ends with an `x-body-length` trailer (`/trailers`), and reports a request's trailers before and after reading its body (`/echo-trailers`).

Full article: [Write-Front and Trailers: Both Ends of a Body](https://behindthepanic.dev/posts/2026-10-05-write-front-and-trailers/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

`[local_server.backends]` in `fastly.toml` declares the `http_me` backend that `/front` uses. Then in another terminal:

```sh
curl http://127.0.0.1:7676/front
curl --raw http://127.0.0.1:7676/front-streaming
curl -X POST --data "hello" http://127.0.0.1:7676/echo-trailers
```

Trailers depend on the protocol and the host. Viceroy doesn't send the `/trailers` trailer at all, and a real Compute service sends it only over HTTP/2 (`curl --http2 -v`, after publishing with `fastly compute publish`). curl can't send a request trailer, so `/echo-trailers` needs a client that can; the post describes what came back.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
