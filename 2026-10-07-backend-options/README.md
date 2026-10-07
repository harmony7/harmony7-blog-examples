# Backend options: tuning and reading back

Reads back every setting of the static backend `http_me` (`/static`), and registers a dynamic backend with nearly every tuning option set and reads those back (`/dynamic`).

Full article: [Backend Options: Tuning and Reading Back](https://behindthepanic.dev/posts/2026-10-07-backend-options/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

`[local_server.backends]` in `fastly.toml` declares `http_me`. Then in another terminal:

```sh
curl http://127.0.0.1:7676/static
curl http://127.0.0.1:7676/dynamic
```

Locally, most of the getters answer `unsupported` ([fastly/Viceroy#723](https://github.com/fastly/Viceroy/issues/723)), and several dynamic backend options aren't applied ([fastly/Viceroy#342](https://github.com/fastly/Viceroy/issues/342)). To see real values, publish it:

```sh
fastly compute publish
```

`service_id` in `fastly.toml` is intentionally blank, so the publish creates a new service, and `[setup.backends]` creates its `http_me` backend.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
