# WebSocket passthrough: handing the connection to a backend

Checks each request for `Upgrade: websocket` and hands WebSocket requests to the `echo` backend (`echo.websocket.org`) with `redirect-to-websocket-proxy`, sending no response of its own. Any other request gets a plain-text reply.

Full article: [WebSocket Passthrough: Handing the Connection Away](https://behindthepanic.dev/posts/2026-09-16-websocket-passthrough/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving
- Node.js, to run [`wscat`](https://www.npmjs.com/package/wscat) through `npx`

## Run it

```sh
fastly compute serve
```

The `echo` backend is defined under `[local_server]` in `fastly.toml`, so there's nothing else to set up. Then in another terminal:

```sh
# The handshake: the 101 and its headers come from the echo server
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Version: 13" -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" \
  http://127.0.0.1:7676/

# A WebSocket session: type a message, and the echo server sends it back
npx wscat -c ws://127.0.0.1:7676/
```

To see what a service without the WebSockets product gets, restart the server with passthrough disabled, and the handoff fails with `unsupported`:

```sh
fastly compute serve --viceroy-args="--enable-local-websocket-passthrough=false"
```

On a real service, the WebSockets product has to be enabled before any handoff succeeds. `[setup.products]` in `fastly.toml` asks the CLI to enable it when `fastly compute publish` first creates the service.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
