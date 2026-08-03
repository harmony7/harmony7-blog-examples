# Hello, world: exporting `http-incoming` by hand

The smallest possible Fastly Compute component: no Rust SDK, no `cargo-component`, just plain `wit-bindgen` against Fastly's own `compute.wit` ABI definition. Ignores the incoming request and body entirely, and just logs `Hello, world!` to the console.

Full article: [Hello, World: Exporting http-incoming by Hand](https://behindthepanic.dev/posts/2026-08-03-hello-world-http-incoming/)

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

`Hello, world!` prints to the `fastly compute serve` console output — not to the HTTP response, which comes back empty (nothing calls `send-downstream`).

## Why `wit/deps` is checked in

`wit/deps/fastly-compute/compute.wit` is a vendored copy of the real Fastly Compute ABI, from the public copy Fastly provides at [fastly/Viceroy](https://github.com/fastly/Viceroy/blob/main/wasm_abi/wit/deps/fastly/compute.wit). It's what `wit-bindgen` reads in `bindings.rs` to generate the bindings the guest needs to call into the Fastly Compute platform.

The `wit/deps/wasi-*` packages are the official WASI 0.2.6 interface definitions, fetched with [`wkg`](https://github.com/bytecodealliance/wasm-pkg-tools). They're needed purely so the WIT resolver can type-check `compute.wit` in full — even the parts (`wasi:filesystem`, `wasi:sockets`) that this example's actual compiled component never ends up importing at runtime. `wit-bindgen` parses the whole package graph, not just the slice you use.

## Why the Rust version is pinned

`rust-toolchain.toml` pins this project to a specific Rust release. That's not incidental: Rust's `wasm32-wasip2` target bakes in whichever WASI Preview 2 point release (`0.2.x`) was current when that version of the compiler shipped, and it drifts forward with every Rust release. Viceroy tolerates a mismatch between that and what `compute.wit` declares; real Fastly Compute's production linker does not — it fails to link with a "missing import" error if the versions don't match exactly. The pinned version here was chosen because it emits WASI `0.2.6`, matching `compute.wit`.
