# Calling a backend

Building an outbound `request` and sending it to a `backend` mirrors building a `response` almost exactly — same static-`new`/instance-method resource shape, just pointed the other direction. This example opens a real backend ([`http-me.fastly.dev`](https://http-me.fastly.dev), a small Fastly Compute app built for HTTP testing), sends it a GET, and reuses the read-loop from [`2026-08-05-reading-the-body`](../2026-08-05-reading-the-body) verbatim to read the response body back.

Full article: [Calling a Backend by Hand](https://behindthepanic.dev/posts/2026-08-06-calling-a-backend-by-hand/)

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

You should get back a JSON description of the request this component sent to `http-me.fastly.dev`, proxied through as the response body.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
