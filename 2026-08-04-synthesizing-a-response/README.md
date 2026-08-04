# Synthesizing a response by hand

Builds on [`2026-08-03-hello-world-http-incoming`](../2026-08-03-hello-world-http-incoming): still no Rust SDK, still plain `wit-bindgen` against `compute.wit`. This time the component actually answers the request — creates a `response`, sets a response header, reads request headers (including a genuinely multi-valued one), writes a body, and sends it downstream.

Includes two small hand-written wrapper functions (`get_header_value`, `get_header_values`) that handle the ABI's `error.buffer-len`-retry and NUL-delimited multi-value parsing — the kind of thing a real SDK would normally do for you.

Full article: [Synthesizing a Response by Hand](https://behindthepanic.dev/posts/2026-08-04-synthesizing-a-response/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

Then in another terminal:

```sh
curl -H "User-Agent: whoever-you-are" -H "Accept: text/html" -H "Accept: application/json" http://127.0.0.1:7676/
```

You should get back:

```
Hello, world! Your user-agent is: whoever-you-are
Accept: ["text/html", "application/json"]
```

with `content-type: text/plain` set explicitly on the response. Send `Accept` as one comma-joined header line instead of two separate `-H` flags, and you'll get back a single value containing the comma — `get-header-values` splits repeated header instances, not comma-separated syntax within one.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
