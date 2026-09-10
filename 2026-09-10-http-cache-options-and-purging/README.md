# HTTP Cache: options and purging

Stores responses through `http-cache` with surrogate keys of your choosing, then purges one of those keys, to show that a key written through `http-cache`'s `write-options` is reachable by `purge-surrogate-key` from a different interface — and that the purge is scoped to that key rather than blunt. `sk=` sets the keys, overriding the suggested ones; `purge=` calls the purge.

Full article: [HTTP Cache: Options and Purging](https://behindthepanic.dev/posts/2026-09-10-http-cache-options-and-purging/)

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

Store two objects under different keys, purge one, and look both up again:

```sh
curl 'https://<your-service>.edgecompute.app/body=alpha?cache=300&ck=a&sk=tagA'
curl 'https://<your-service>.edgecompute.app/body=beta?cache=300&ck=b&sk=tagB'
curl 'https://<your-service>.edgecompute.app/purge=tagA'
```

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
