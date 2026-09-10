# Core Cache: insert and replace

Writes a value under the key `core-cache-demo` with `insert`, then reaches the same object again through the separate `replace` API and reports what each call returns. Shows the two write paths side by side, including which parts of `replace` Viceroy does not implement.

Full article: [Core Cache: Insert and Replace](https://behindthepanic.dev/posts/2026-08-26-core-cache-insert-replace/)

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
