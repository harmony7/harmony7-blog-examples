# HTTP Cache: serving stale content

Shows what the cache means by a stale object that is still usable. A `ck=` directive puts several requests on one cache key so a response stored by one can be revalidated by another, and `fail=1` skips the backend send entirely, standing in for a before-send hook that returns an error, so `transaction-choose-stale` becomes reachable.

Full article: [HTTP Cache: Serving Stale Content](https://behindthepanic.dev/posts/2026-09-09-http-cache-serving-stale-content/)

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

Seed an entry with an error window, wait for it to go stale, then fail the send:

```sh
curl 'https://<your-service>.edgecompute.app/body=x?header=Cache-Control%3Amax-age%3D1%2Cstale-if-error%3D60&ck=demo'
curl 'https://<your-service>.edgecompute.app/anything?ck=demo&fail=1'
```

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
