# http-downstream: who is asking

Prints what the platform knows about the client behind this request — its IP address, the request id, the original header names and their order, the OH fingerprint, and whether the connection was flagged by DDoS detection.

Full article: [http-downstream: Who's Actually Asking](https://behindthepanic.dev/posts/2026-08-21-http-downstream-identity/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

Then in another terminal:

```sh
curl http://127.0.0.1:7676/
```

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
