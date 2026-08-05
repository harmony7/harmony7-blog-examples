# Reading the body by hand

Builds on [`2026-08-04-synthesizing-a-response`](../2026-08-04-synthesizing-a-response): still no Rust SDK. This time the component reads the incoming request body — in a loop, in caller-chosen chunks, checking `is_ready()` first — and echoes back how many bytes it read and what they were.

See also [`2026-08-05-body-passthrough`](../2026-08-05-body-passthrough), a companion example that skips reading the body entirely and hands it straight to the outgoing response.

Full article: [The Body Isn't Part of the Request (or the Response)](https://behindthepanic.dev/posts/2026-08-05-the-body-isnt-part-of-the-request/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

Then in another terminal:

```sh
curl -X POST -d "hello from the request body" http://127.0.0.1:7676/
```

You should get back `You sent 27 bytes: hello from the request body`, and the `fastly compute serve` console should show `request body ready before first read: true` followed by the read confirmation.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
