# Pending responses: editing headers you'll never hold

Starts a backend request with `send-async`, queues header changes on the `pending-response` for a real response, for the synthetic 5XX Compute makes up if the request fails, and for both, then hands it off with `send-downstream-pending` and returns without waiting. `/ok` removes an `x-internal` header the backend sets, `/fail` sends to a name that can't resolve, and `/slow` waits two seconds on the backend while the program finishes immediately.

Full article: [Pending Responses: Editing Headers You'll Never Hold](https://behindthepanic.dev/posts/2026-10-06-pending-response-headers/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

`[local_server.backends]` in `fastly.toml` declares the `http_me` backend, so there's nothing else to set up. Then in another terminal:

```sh
curl -i http://127.0.0.1:7676/ok
curl -i http://127.0.0.1:7676/fail
curl -i http://127.0.0.1:7676/slow
```

For `/slow`, compare how long curl waits with the "guest completed" line in the `fastly compute serve` console. Viceroy's 502 for `/fail` has its own error text as the body, where a real service answers `Bad Gateway` ([fastly/Viceroy#729](https://github.com/fastly/Viceroy/issues/729)).

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
