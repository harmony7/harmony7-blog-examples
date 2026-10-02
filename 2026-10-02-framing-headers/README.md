# Framing headers: automatic and manual `Content-Length`, decompression, and keepalive

Answers a different path for each case: a six-byte body sent with automatic framing and a wrong `Content-Length`, the same body with manual framing and a correct, short, long, or invalid `Content-Length`, a streamed response, a fetch of http-me's gzipped JSON with and without automatic decompression, and a request for `no-keepalive`.

Full article: [Framing Headers: Who Decides Where a Body Ends](https://behindthepanic.dev/posts/2026-10-02-framing-headers/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

`[local_server.backends]` in `fastly.toml` declares the `http_me` backend, so there's nothing else to set up. Then in another terminal:

```sh
# Framing only matters in HTTP/1.1; --raw shows chunk sizes
curl -i --raw --http1.1 http://127.0.0.1:7676/auto
curl -i --raw --http1.1 http://127.0.0.1:7676/manual-short
curl -i --raw --http1.1 http://127.0.0.1:7676/stream

# Decompression, and the address the backend response came from
curl http://127.0.0.1:7676/gzip
```

The other paths are `/manual`, `/manual-long` (use `-m 5`, since the client waits for bytes that never arrive), `/manual-invalid`, `/accept-encoding`, and `/no-keepalive`.

Several of these behave differently on a real Compute service, which is what the post compares. To try that, publish it:

```sh
fastly compute publish
```

`service_id` in `fastly.toml` is intentionally blank, so the publish creates a new service, and `[setup.backends]` creates its `http_me` backend.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
