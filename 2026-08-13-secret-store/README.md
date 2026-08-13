# Secret Store

Opens a Secret Store named `creds` and reads back a key (`api-token` by default, or whatever's sent in an `x-key` header). Demonstrates that `store.get` hands back a `secret` resource, not the value itself — the plaintext bytes only come out through a second call, `secret.plaintext`.

Full article: [Secret Store: Configuration You Don't Want in a Log Line](https://behindthepanic.dev/posts/2026-08-13-secret-store-configuration-you-dont-want-in-a-log-line/)

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
curl -H "x-key: nope" http://127.0.0.1:7676/
```

Local test data for the store lives directly in `fastly.toml` under `[local_server.secret_stores.creds]`; the real store's contents are managed outside Compute entirely, same as Config Store.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
