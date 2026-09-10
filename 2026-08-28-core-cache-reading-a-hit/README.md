# Core Cache: reading a hit

Stores the alphabet twice, once with a declared length and once without, then reads both back to show what the cache can tell you about an object it is still receiving. The declared-length case can report its size immediately; the other cannot.

Full article: [Core Cache: Reading a Hit](https://behindthepanic.dev/posts/2026-08-28-core-cache-reading-a-hit/)

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
