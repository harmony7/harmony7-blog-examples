# Shielding: forwarding to a shield POP by hand

Reads `shield-info` for the shield code `pdx-or-us`, parses the flag byte and targets out of the string it returns, and either builds a backend with `backend-for-shield` and forwards the request to the shield, or sends it straight to the `origin` backend. Each hop writes a short report, so one response shows every hop the request took.

Full article: [Shielding: A Record Wearing a String](https://behindthepanic.dev/posts/2026-09-15-shielding/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

The example runs as two local instances of the same program. `fastly.toml` makes one of them an edge POP, and `fastly.shield.toml` makes the other the `pdx-or-us` shield on port 7677. Both manifests define the `origin` backend under `[local_server]`, so there's nothing else to set up.

```sh
fastly compute serve --env=shield --addr=127.0.0.1:7677
```

Then in another terminal:

```sh
fastly compute serve
```

And in a third:

```sh
# Lands on the edge, which forwards to the shield, which goes to origin
curl http://127.0.0.1:7676/body=hello-from-origin

# Lands on the shield directly, so the program runs once
curl http://127.0.0.1:7677/body=hello-from-origin

# A shield code neither manifest knows: shield-info fails, and the request goes straight to origin
curl -H "shield: waffle-cone" http://127.0.0.1:7676/body=hello-from-origin
```

The path is passed through to [http-me](https://http-me.fastly.dev), so `/body=...` sets what the origin replies with.

Both shield URLs in `fastly.toml` are plain HTTP, including the `encrypted` one, because Viceroy's shield backend failed certificate validation against an HTTPS server.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
