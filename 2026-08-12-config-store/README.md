# Config Store

Opens a Config Store named `settings` and reads back a key (`greeting` by default, or whatever's sent in an `x-key` header), demonstrating that Config Store is a read-only-from-Compute key/value lookup — there's no `set`/`insert` in the WIT interface at all.

Full article: [Config Store: Configuring a Compute App](https://behindthepanic.dev/posts/2026-08-12-config-store-configuring-a-compute-app/)

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
curl -H "x-key: site.name" http://127.0.0.1:7676/
curl -H "x-key: does-not-exist" http://127.0.0.1:7676/
```

Local test data for the store lives directly in `fastly.toml` under `[local_server.config_stores.settings]`; the real store's contents are managed outside Compute entirely (Fastly's UI/API), which is the whole point of the article.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
