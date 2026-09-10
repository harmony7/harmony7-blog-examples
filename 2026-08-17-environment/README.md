# Environment: reading the host through WASI, not Fastly

Lists every `FASTLY_*` environment variable the host exposes, then calls `get-arguments()` and `initial-cwd()` to show what a Compute guest gets for the two other things `wasi:cli/environment` offers. Nothing here is Fastly-specific: it is the standard WASI interface, and Fastly's values simply arrive through it.

Full article: [Environment: The Interface That Isn't Fastly-Specific At All](https://behindthepanic.dev/posts/2026-08-17-environment/)

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

The variable list comes back in the HTTP response.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
