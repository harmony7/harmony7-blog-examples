# Dynamic backends

`backend.open` only finds backends already declared ahead of time (in `fastly.toml` locally, or the Fastly service config in production). This example does the opposite: it takes a hostname from the incoming request's `x-target-host` header (falling back to [`http-me.fastly.dev`](https://http-me.fastly.dev) if the header isn't set), and registers a `backend` for it at runtime via `register-dynamic-backend`, with no entry in `fastly.toml` at all.

Full article: [Dynamic Backends by Hand](https://behindthepanic.dev/posts/2026-08-07-dynamic-backends-by-hand/)

> [!WARNING]
> This example takes `x-target-host` straight from the request with no validation, which is an [SSRF vulnerability](https://developer.mozilla.org/en-US/docs/Web/Security/Attacks/SSRF). It's left unfixed here to keep the demo focused on `register-dynamic-backend` — don't ship this pattern as-is. Validate request-derived input against a set of hosts you've decided are safe to reach before passing it to `register-dynamic-backend`.

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

Or point it somewhere else entirely, with no config change:

```sh
curl -H "x-target-host: http-me.fastly.dev" http://127.0.0.1:7676/
```

You should get back a JSON description of the request this component sent to whichever host you named, plus `is_dynamic: true` confirming the backend it just built isn't one of the static, ahead-of-time ones.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
