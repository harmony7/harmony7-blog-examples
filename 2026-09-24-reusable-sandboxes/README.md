# Reusable sandboxes: one sandbox, more than one request

Answers a request, then calls `next-request` and `await-request` to wait up to five seconds for another, in a loop. Each response reports the sandbox ID, the request ID, and how many requests that sandbox has handled, counted in a `static` that is never reset.

Full article: [Reusable Sandboxes: A Sandbox Doesn't Have to Be for Exactly One Request](https://behindthepanic.dev/posts/2026-09-24-reusable-sandboxes/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

Then in another terminal:

```sh
for i in $(seq 1 8); do curl -s http://127.0.0.1:7676/; done
```

Viceroy gives a sandbox up to five requests after its first, so the first six responses share a sandbox ID and a climbing count, and the seventh starts a new sandbox. Wait longer than five seconds before the next request and the waiting sandbox gives up, so that request also gets a fresh one. When a sandbox is done, the program prints why to the terminal running the server.

On a real service, reuse needs a service-level setting turned on, and even then it depends on the next request reaching the same cache node, so don't expect the same neat sequence.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
