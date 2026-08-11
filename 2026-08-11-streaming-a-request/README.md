# Streaming a request

Forwards the incoming request body to [`http-me.fastly.dev`](https://http-me.fastly.dev) via `send-async-streaming`, reading and forwarding it a few bytes at a time (with an artificial pause between chunks standing in for a slow client upload) instead of buffering the whole thing before sending anything. Logs a timestamp at each step to show that the pending response doesn't come back until the streamed body is actually finished.

Full article: [Streaming a Request, by Hand](https://behindthepanic.dev/posts/2026-08-11-streaming-a-request-by-hand/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

Then in another terminal:

```sh
curl -X POST -d "hello from the request body, streamed in pieces" http://127.0.0.1:7676/
```

You should see `send-async-streaming` return at `+0.00s`, several "forwarded" lines ticking up roughly every 250ms as each chunk goes out, then `close`, and only after that does `await-response` resolve.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
