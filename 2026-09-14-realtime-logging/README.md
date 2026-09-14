# Realtime Logging: the function that can't fail

Opens logging endpoints under several names and writes one event to each: `my_endpoint`, a name that was never configured, and names that break the doc comment's rules (empty, containing a colon, containing a newline, and the reserved `stdout` and `stderr`). The response reports what `open` returned for each. It also prints a line with `println!` for contrast, which lands in the same terminal as the log events.

Full article: [Realtime Logging: The Function That Can't Fail](https://behindthepanic.dev/posts/2026-09-14-realtime-logging/)

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

The response lists what `open` returned for each name. The events themselves, and the `println!` line, appear in the terminal running `fastly compute serve`, whatever name they were written to, since nothing needs configuring locally.

On a real service, events only go somewhere if a logging endpoint with that name exists. Create one (for example with `fastly logging https create --name=my_endpoint ...`; see Fastly's [developer guide to third-party logging](https://www.fastly.com/documentation/guides/integrations/non-fastly-services/developer-guide-logging/) for the supported endpoint types) and activate the version; `[setup.log_endpoints]` in `fastly.toml` records that the package expects it.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
