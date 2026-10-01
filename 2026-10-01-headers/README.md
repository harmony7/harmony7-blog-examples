# Headers: listing, writing, and removing them

Lists the incoming request's header names through a deliberately small 24-byte buffer so the cursor has to page, then builds a response with `append-header` (two `Set-Cookie` values), `set-header-values` (with and without the final NUL), and `remove-header` (on a header that exists and one that doesn't), sets a `207` status, and reports all of it in the response body.

Full article: [Headers: Listing, Writing, and Removing Them](https://behindthepanic.dev/posts/2026-10-01-headers/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

Then in another terminal:

```sh
curl -i -H "Accept: text/plain" -H "X-Custom-Thing: hi" http://127.0.0.1:7676/

# The request version the example reports changes with the client's
curl --http1.0 http://127.0.0.1:7676/
```

Use `-i` to see the response headers themselves: two separate `set-cookie` lines, three `x-color` lines, and one `x-shape`.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
