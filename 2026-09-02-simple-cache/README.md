# Simple Cache, built by hand

Reimplements the Simple Cache convenience layer on top of the Core Cache: derives the same cache keys for a `greeting` entry, including the POP-scoped variant that uses `compute-runtime`'s POP name, and stores through `transaction-insert-and-stream-back`.

Full article: [Simple Cache, Built by Hand](https://behindthepanic.dev/posts/2026-09-02-simple-cache/)

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
