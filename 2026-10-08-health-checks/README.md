# Health checks

Breaks each of `healthcheck-options`' documented rules to see which ones are enforced (`/rules`), and registers two health-checked dynamic backends and watches `is-healthy` for six seconds (`/health`).

Full article: [Health Checks: Rules the Defaults Break](https://behindthepanic.dev/posts/2026-10-08-health-checks/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

Both paths use dynamic backends, so `fastly.toml` declares none. Then in another terminal:

```sh
curl http://127.0.0.1:7676/rules
curl http://127.0.0.1:7676/health
```

Locally, the health checks are accepted but never run, so `is-healthy` stays `unknown`. To try it on the real platform, publish it:

```sh
fastly compute publish
```

`service_id` in `fastly.toml` is intentionally blank, so the publish creates a new service. Health checks on dynamic backends aren't available on every service, and `/health` reports `unsupported` where they aren't.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
