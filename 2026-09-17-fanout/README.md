# Fanout: handing a request to the GRIP proxy

Hands every request to the `origin` backend through the GRIP proxy with `redirect-to-grip-proxy`. The origin, `origin.mjs`, answers in GRIP: it tells the proxy to hold the connection open as an HTTP stream subscribed to the channel `test`, so anything published to `test` afterward arrives on that connection.

Full article: [Fanout: Letting Your Origin Decide What a Connection Becomes](https://behindthepanic.dev/posts/2026-09-17-fanout/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving
- [Pushpin](https://pushpin.org/docs/install/), on your `PATH`. The CLI starts it for you because `[local_server.pushpin]` is enabled in `fastly.toml`. If a Pushpin service is already running on the machine, stop it first, since it holds the same ports.
- Node.js, for `origin.mjs`

## Run it

Start the stand-in origin:

```sh
node origin.mjs
```

Then, in another terminal, start Pushpin and the local server together:

```sh
fastly compute serve
```

Open a connection. It stays open:

```sh
curl -i -N http://127.0.0.1:7676/
```

And from a third terminal, publish to the channel:

```sh
curl -d '{"items":[{"channel":"test","formats":{"http-stream":{"content":"hello from a publisher\n"}}}]}' \
  http://127.0.0.1:5561/publish/
```

The message appears on the open connection. The CLI writes Pushpin's logs to `pushpin-logs/` in this directory.

On a real service, `[setup]` points at an origin placeholder you'll need to replace with one that answers in GRIP, and asks the CLI to enable Fanout when `fastly compute publish` first creates the service. Publishing then goes to `https://api.fastly.com/service/{SERVICE_ID}/publish/` with an API token in `Fastly-Key`.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
