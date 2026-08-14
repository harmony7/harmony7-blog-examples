# KV Store

Opens a KV Store named `notes` and, in a single request, inserts a value, looks it up (reading its body, generation, and metadata), deletes it, and looks it up again to confirm it's gone — key and value configurable via an `x-key` header and the request body.

Uses the `*-async`/`await-*` KV Store functions rather than the plain `lookup`/`insert`/`delete`, because Viceroy 0.20.1's Wasm Component support doesn't yet implement the blocking variants (see [fastly/Viceroy#657](https://github.com/fastly/Viceroy/pull/657), open as of writing) — calling them returns `KvError::InternalError` regardless of arguments. The async versions work correctly and are the ones actually exercised here.

Full article: [KV Store: Read and Write at the Edge](https://behindthepanic.dev/posts/2026-08-14-kv-store-read-and-write-at-the-edge/)

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
curl -H "x-key: my-note" -d "a value I wrote myself" http://127.0.0.1:7676/
```

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
