# What the world leaves out

A Compute service with two Cargo features that each add a standard-library call importing a WASI interface Compute's `service` world doesn't include: `fs` (`wasi:filesystem`) and `net` (`wasi:sockets`). `no-files/` is a second component that exports `wasi:filesystem` with nothing in it. The build composes the two with `wac plug`, so the `fs` build's imports all land inside the `service` world.

Full article: [What the World Leaves Out](https://behindthepanic.dev/posts/2026-10-20-what-the-world-leaves-out/)

## Prerequisites

- Rust with the `wasm32-wasip2` and `wasm32-unknown-unknown` targets (`rust-toolchain.toml` installs both)
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving
- [`wasm-tools`](https://github.com/bytecodealliance/wasm-tools), to turn `no-files` into a component and to list a component's imports
- [`wac`](https://github.com/bytecodealliance/wac), to compose the two components

## Run it

```sh
fastly compute serve
curl http://127.0.0.1:7676/
```

The response shows `read_to_string` failing with `NotFound`, from inside a handler that ran.

To see the uncomposed build fail instead, build the service alone and serve it:

```sh
cargo build -p service --features fs
mkdir -p bin && cp target/wasm32-wasip2/debug/service.wasm bin/main.wasm
fastly compute serve --skip-build
```

## Why `no-files` targets `wasm32-unknown-unknown`

Built for `wasm32-wasip2`, the component imports `wasi:filesystem/types` even though its code never calls it. The import comes from Rust's linker, `wasm-component-ld`, which attaches a WASI preview1 adapter while it builds the component, and the adapter's import of that interface survives. The `no-files` component would then import the interface it's trying to provide, and the composed component would still need a filesystem from the host. On `wasm32-unknown-unknown`, there's no WASI and no adapter, and `wasm-tools component new` turns the core module into a component.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
