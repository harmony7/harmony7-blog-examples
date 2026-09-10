# http-downstream: what kind of connection this is

Prints the TLS facts about the current connection — protocol, cipher suite, JA3 and JA4 fingerprints, the raw ClientHello — alongside the bot-detection verdict: whether a bot was detected, what it was named and categorized as, and whether the request was analyzed at all.

Full article: [http-downstream: What Kind of Connection Is This, Really](https://behindthepanic.dev/posts/2026-08-24-http-downstream-tls-bots/)

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

Most TLS fields are empty over plain HTTP locally; the shapes are still visible.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
