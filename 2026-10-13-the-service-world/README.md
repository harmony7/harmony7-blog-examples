# The service world

A handler that imports only what Compute's `service` world provides. The `proxy-world/` crate is a separate component that targets `wasi:http/proxy` instead of Compute's world, and fails to start because Compute doesn't provide `wasi:http`.

Full article: [The service World: WASI Without wasi:http](https://behindthepanic.dev/posts/2026-10-13-the-service-world/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving
- [`wasm-tools`](https://github.com/bytecodealliance/wasm-tools), to list a built component's imports

## Run it

```sh
fastly compute serve
curl http://127.0.0.1:7676/
```

To see what the component imports, build it and ask `wasm-tools`:

```sh
cargo build --target wasm32-wasip2
wasm-tools component wit target/wasm32-wasip2/debug/the_service_world.wasm
```

`proxy-world/` has its own `fastly.toml`. Run `fastly compute serve` there to see Viceroy refuse it.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged. `proxy-world/wit/deps` adds the official `wasi:http` 0.2.6 package.
