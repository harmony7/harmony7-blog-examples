# The same tally, in JavaScript

The service from the 2026-10-16 example, unchanged, composed with a `tally` component written in JavaScript. `service/` is the Rust Compute service. `tally/tally.js` is the JavaScript `tally`, built into a component with [`jco componentize`](https://github.com/bytecodealliance/jco). `no-files/` is the component from the 2026-10-20 example, which supplies the `wasi:filesystem` imports the JavaScript engine brings. The build plugs `no-files` into `tally`, then `tally` into the service.

Full article: [The Same Tally, in JavaScript](https://behindthepanic.dev/posts/2026-10-23-a-tally-in-javascript/)

## Prerequisites

- Rust with the `wasm32-wasip2` and `wasm32-unknown-unknown` targets: `rustup target add wasm32-wasip2 wasm32-unknown-unknown`
- Node.js and npm, then `npm install` in `tally/` for `jco` and `componentize-js`
- [`wasm-tools`](https://github.com/bytecodealliance/wasm-tools) and [`wac`](https://github.com/bytecodealliance/wac), which the build script uses
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
(cd tally && npm install)
fastly compute serve
curl http://127.0.0.1:7676/
```

The response is `bytes reached 42`, and Viceroy's log shows each `tally` being created and dropped.

## It runs in Viceroy only

A real Compute service rejects this component at upload: it contains three Wasm linear memories (the service, the JavaScript engine, and `no-files`), and Compute allows two per component. The article covers why.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
