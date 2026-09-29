# Next-Gen WAF: inspecting a request before forwarding it

Calls `security.inspect` on the incoming request, parses the JSON verdict, and answers with the WAF's suggested status unless the verdict is `allow`. Allowed requests are forwarded, as the same request and body that were inspected, to `http-me.fastly.dev`. Before that, it makes two calls that should fail: one without a workspace, and one with a buffer too small for the verdict.

Full article: [Next-Gen WAF: A Verdict You Have to Act On](https://behindthepanic.dev/posts/2026-09-29-next-gen-waf/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

The `origin` backend is defined under `[local_server]` in `fastly.toml`. Then in another terminal:

```sh
curl http://127.0.0.1:7676/
curl "http://127.0.0.1:7676/?id=1%27%20OR%20%271%27=%271"
```

Both come back `allow`. Viceroy's verdict is a constant, so the block and redirect paths can't be reached locally.

On a real service, a verdict needs four things: the Next-Gen WAF enabled on the service, a linked workspace, blocking mode, and rules. `[setup.products.ngwaf]` in `fastly.toml` enables the product and links a workspace when `fastly compute publish` first creates the service. Replace `YOUR_WORKSPACE_ID` first, and replace the `my-corp` and `my-workspace-id` placeholders in `src/lib.rs` with your corp's short name and your workspace's ID.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
