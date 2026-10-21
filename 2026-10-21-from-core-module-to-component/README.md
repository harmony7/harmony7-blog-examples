# From core module to component

A Compute service whose handler reads the request's URI with `get-uri` and sends it back. `stages.sh` builds the same source a second way, one stage at a time, leaving the core module (`target/core.wasm`) and the component made from it (`target/component.wasm`) side by side to inspect.

Full articles: [The Module Before the Component](https://behindthepanic.dev/posts/2026-10-21-the-module-before-the-component/), on the core module, and [From Core Module to Component](https://behindthepanic.dev/posts/2026-10-22-from-core-module-to-component/), on what the second stage builds from it

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving
- [`wasm-tools`](https://github.com/bytecodealliance/wasm-tools), for `stages.sh` and for looking inside the results

## Run it

```sh
fastly compute serve
```

Then in another terminal:

```sh
curl http://127.0.0.1:7676/some/path
```

The response is `uri: http://127.0.0.1:7676/some/path`.

## Build it in two stages

```sh
./stages.sh
wasm-tools print target/core.wasm | grep -E '^  \((import|export)'
wasm-tools print target/component.wasm | less
```

`target/component.wasm` is for reading, not deploying. `stages.sh` runs `wasm-tools component new` without the WASI adapter that Rust's linker attaches, so its WASI imports come out at `@0.2.4` instead of `@0.2.6`. Viceroy runs it, but a real Compute service refuses it at upload. `fastly compute serve` and `fastly compute publish` use the normal one-command build, which is unaffected.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
