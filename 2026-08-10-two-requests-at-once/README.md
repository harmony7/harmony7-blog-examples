# Two requests at once

Sends two GET requests to [`http-me.fastly.dev`](https://http-me.fastly.dev) at the same time via `send-async`, each with a different artificial delay (`http-me`'s `wait=<ms>` directive), then polls both with `select-with-timeout` in a loop and reports each one the moment it finishes — not in the order they were sent.

Full article: [Two Requests at Once, by Hand](https://behindthepanic.dev/posts/2026-08-10-two-requests-at-once-by-hand/)

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

The "slow" request is set to a 3-second delay and the "fast" one to 1 second. You should see the fast one reported at around the 1-second mark, periodic "still waiting" lines from the `select-with-timeout` poll loop, and the slow one reported at around 3 seconds — with total elapsed time close to 3 seconds, not 4, since both requests were in flight the whole time.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
