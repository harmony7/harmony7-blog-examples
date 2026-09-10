# Purging by surrogate key

Stores several objects tagged `product-42`, `product-42-en`, `product-42-fr` and `catalog`, purges one key, and looks all of them up again to show exactly which objects a single purge reaches. Also derives a surrogate key the way Fastly's SDKs do, to check the derivation is interoperable.

Full article: [Purging by Surrogate Key](https://behindthepanic.dev/posts/2026-09-01-surrogate-key-purging/)

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
