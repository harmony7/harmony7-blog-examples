# Resources aren't host-only

Two components. `tally/` exports a `tally` resource that it implements itself. `service/` is a Compute service that imports `tally` and uses it through handles, the way it uses any host resource. `wac plug` composes them into one component whose imports are all in Compute's `service` world.

Full article: [Resources Aren't Host-Only](https://behindthepanic.dev/posts/2026-10-16-guest-owned-resources/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving
- [`wac`](https://github.com/bytecodealliance/wac), which the build script uses to compose the two components

## Run it

```sh
fastly compute serve
curl http://127.0.0.1:7676/
```

Viceroy's log shows each `tally` being created and dropped, including the drop that runs inside `tally` when the service drops its handle.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
