# Device Detection: what the client is running

Reads the incoming `User-Agent` header, hands it to `device-detection.lookup`, and prints the record that comes back. Send different user agents to see the fields change.

Full article: [Device Detection: What are Your Visitors Using to Reach You?](https://behindthepanic.dev/posts/2026-08-19-device-detection/)

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

Try a real user agent to get a populated record:

```sh
curl http://127.0.0.1:7676/ -H 'User-Agent: Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)'
```

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
