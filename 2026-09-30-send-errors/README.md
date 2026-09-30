# Send errors: reading `error-with-detail` instead of discarding it

Registers one dynamic backend per failure case (an unresolvable name, a reserved address, a port that never answers, an expired certificate, a TLS version cap, and a too-slow origin), sends a `GET` through each with `send-uncached`, and prints the `error` and `send-error-detail` that come back, one line per case.

Full article: [Send Errors: Asking a Failed Backend Call What Went Wrong](https://behindthepanic.dev/posts/2026-09-30-send-errors/)

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

The report comes back in the response body. Locally, every failure reads `detail: none`: Viceroy doesn't populate `send-error-detail` yet ([fastly/Viceroy#727](https://github.com/fastly/Viceroy/issues/727)). To see real detail values, publish it to a Compute service:

```sh
fastly compute publish
```

`service_id` in `fastly.toml` is intentionally blank, so the publish creates a new service rather than touching an existing one. There's nothing else to set up, because every backend is registered at runtime.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
