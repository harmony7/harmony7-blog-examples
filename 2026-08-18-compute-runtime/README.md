# Compute Runtime: the typed half of the same data

Calls every getter on `compute-runtime` in one request — POP, region, hostname, customer id, namespace id, cache generation, heap size, staging flag — and prints each with its return type. The same facts the environment variables carry, but typed rather than stringly.

Full article: [Compute Runtime: Typed Access to Environment Data](https://behindthepanic.dev/posts/2026-08-18-compute-runtime/)

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
