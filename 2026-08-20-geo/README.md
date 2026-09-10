# Geo: what the platform knows about an IP

Looks up `8.8.8.8` with `geo.lookup` and prints the whole record. A fixed address rather than the client's, so the output is the same every run and the fields are easy to compare against the WIT.

`fastly.toml` configures `[local_server.geolocation]`, so Viceroy answers the lookup locally with no setup.

Full article: [Geo: And Where are Your Visitors From?](https://behindthepanic.dev/posts/2026-08-20-geo/)

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

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
