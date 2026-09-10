# Core Cache: transactions

Runs `transaction-lookup` against the key `transaction-demo` and prints the returned lookup state one flag at a time, so the obligation the cache hands back is visible rather than inferred. Then discharges it and looks up again.

Full article: [Core Cache: Transactions](https://behindthepanic.dev/posts/2026-08-27-core-cache-transactions/)

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
