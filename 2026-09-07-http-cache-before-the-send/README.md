# HTTP Cache: before the send

Calls the four `http-cache` functions that run before you fetch anything — `is-request-cacheable`, `get-suggested-cache-key`, `transaction-lookup`, and `get-suggested-backend-request` — and prints what each returns. The lookup runs twice, once with `backend: none` and once with a real backend handle, to settle what that option actually changes.

Full article: [HTTP Cache: Before the Send](https://behindthepanic.dev/posts/2026-09-07-http-cache-before-the-send/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`)
- A Fastly account, because this one does not run locally

## Run it

Every `http-cache` function returns `Error::Unsupported` under Viceroy, so this example only does anything against a real service:

```sh
fastly compute publish
```

`service_id` is deliberately blank, so publishing creates a new service rather than touching an existing one, and the `http_me` backend is created from `[setup]` in `fastly.toml`. Running it locally with `fastly compute serve` builds and serves fine; every cache call just reports `Unsupported`.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
