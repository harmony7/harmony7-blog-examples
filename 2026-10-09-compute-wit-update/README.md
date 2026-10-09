# The September 2026 `compute.wit` update: `clone` and a cache lookup timeout

Clones a log endpoint and a backend and uses each clone after dropping the original (`/clone-log`, `/clone-backend`), and sends two requests at once for the same slow, cacheable URL so the second collapses onto the first, optionally with a lookup timeout set through `extra-cache-override-details` (`/collapse`).

Full article: [compute.wit Changed: Filling In an extra](https://behindthepanic.dev/posts/2026-10-09-compute-wit-update/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

`[local_server.backends]` in `fastly.toml` declares `http_me`. Then in another terminal:

```sh
curl http://127.0.0.1:7676/clone-backend
curl http://127.0.0.1:7676/clone-log

# Use a new run= value each time, so nothing is cached yet
curl "http://127.0.0.1:7676/collapse?run=1"
curl "http://127.0.0.1:7676/collapse?run=2&timeout=500"
```

Locally, `/clone-log` fails with a 500: Viceroy's `endpoint.clone` traps ([fastly/Viceroy#731](https://github.com/fastly/Viceroy/issues/731)). And `/collapse` shows nothing, because Viceroy has no HTTP cache for the requests to collapse in. To see the lookup timeout work, publish it:

```sh
fastly compute publish
```

`service_id` in `fastly.toml` is intentionally blank, so the publish creates a new service, and `[setup.backends]` creates its `http_me` backend.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
