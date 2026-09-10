# HTTP Cache: after the send

Runs the whole guest cache round trip and reports which storage action came back and which function discharged the obligation. The incoming path is passed straight through to the origin, so the URL you request picks the origin's behavior and therefore the branch: a cacheable response inserts, a stale one revalidates and updates, `no-store` records uncacheable, and a 500 abandons.

Full article: [HTTP Cache: After the Send](https://behindthepanic.dev/posts/2026-09-08-http-cache-after-the-send/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`)
- A Fastly account, because this one does not run locally

## Run it

Every `http-cache` function returns `Error::Unsupported` under Viceroy, so this example only does anything against a real service:

```sh
fastly compute publish
```

`service_id` is deliberately blank, so publishing creates a new service rather than touching an existing one, and the `http_me` backend is created from `[setup]` in `fastly.toml`. Running it locally with `fastly compute serve` builds and serves fine; every cache call just reports `Unsupported`.

Some examples of driving it, once published:

```sh
curl https://<your-service>.edgecompute.app/body=hello?cache=60
curl https://<your-service>.edgecompute.app/no-cache
curl https://<your-service>.edgecompute.app/status=500
```

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
