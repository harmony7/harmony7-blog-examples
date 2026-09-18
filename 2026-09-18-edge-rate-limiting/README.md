# Edge rate limiting: a rate counter and a penalty box by client IP

Opens a rate counter named `requests_by_ip` and a penalty box named `banned_ips`, calls `check-rate` for the client's IP with a limit of 5 requests per second over a 10-second window and a 60-second penalty, and answers `429` if the client is penalized. Every response also reports what the lookups returned, plus three calls the doc comments say should be rejected.

Full article: [Edge Rate Limiting: The Counter That Always Says Zero](https://behindthepanic.dev/posts/2026-09-18-edge-rate-limiting/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

Then in another terminal:

```sh
for i in $(seq 1 30); do curl -s -o /dev/null -w "%{http_code} " http://127.0.0.1:7676/; done
curl http://127.0.0.1:7676/
```

Every request gets a `200`, and every lookup reports zero. That's expected: Viceroy implements the `erl` functions as constants, so nothing is counted locally. Only a real service limits anything, and there's nothing to enable or configure there first. Counters and penalty boxes are created by opening them by name.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
