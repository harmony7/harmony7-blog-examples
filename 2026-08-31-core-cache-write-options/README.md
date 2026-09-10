# Core Cache: write options

Inserts under `write-options-demo` with a vary rule of `accept-encoding user-agent`, surrogate keys `catalog product-42`, and an explicit length, then reads every option back off the stored object. A second insert under `write-options-sensitive` sets the sensitive-data flag to see what changes from inside the guest.

Full article: [Core Cache: Write Options](https://behindthepanic.dev/posts/2026-08-31-core-cache-write-options/)

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
