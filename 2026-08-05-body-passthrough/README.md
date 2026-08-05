# Body passthrough

A body doesn't belong to the request or response it arrived with — it's an independent resource. This example takes that as far as it goes: it hands the *incoming* request body straight to `send-downstream` as the *outgoing* response body, without ever reading a byte of it.

See also [`2026-08-05-reading-the-body`](../2026-08-05-reading-the-body), a companion example that actually reads the body's content by hand instead of passing it through.

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
curl -X POST -d "echo this back verbatim" http://127.0.0.1:7676/
```

You should get back exactly `echo this back verbatim` — the request body, unmodified, sent back out as the response body.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
