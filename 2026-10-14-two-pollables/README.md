# Two pollables

Waits on two `wasi:clocks` timers through `wasi:io/poll` (`/timer`), and on a slow backend request through `fastly:compute/async-io` (`/backend`). The `mix` feature tries to pass both kinds of `pollable` to one `select`, and doesn't compile.

Full article: [Two Pollables, One World](https://behindthepanic.dev/posts/2026-10-14-two-pollables/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

`[local_server.backends]` in `fastly.toml` declares `http_me`. Then in another terminal:

```sh
curl http://127.0.0.1:7676/timer
curl http://127.0.0.1:7676/backend
```

To see the type error, build with the `mix` feature:

```sh
cargo build --target wasm32-wasip2 --features mix
```

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
