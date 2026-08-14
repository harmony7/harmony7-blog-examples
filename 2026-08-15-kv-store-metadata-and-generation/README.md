# KV Store: Metadata and Generation

Opens a KV Store named `flags` and, in a single request, walks through an optimistic-concurrency dance on one key, `beta-checkout`: seeds a value, reads its metadata and generation back without ever calling `take-body`, makes a conditional write using that generation (succeeds), then makes a second conditional write reusing the same now-stale generation (rejected with `kv-error.precondition-failed`) — then looks the key up one more time, body included, to confirm which write actually won.

Uses the `*-async`/`await-*` KV Store functions rather than the plain `lookup`/`insert`, for the same reason as the previous KV Store example: Viceroy 0.20.1's Wasm Component support doesn't yet implement the blocking variants (see [fastly/Viceroy#657](https://github.com/fastly/Viceroy/pull/657), open as of writing).

Full article: [KV Store: Metadata and Generation](https://behindthepanic.dev/posts/2026-08-15-kv-store-metadata-and-generation/)

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
