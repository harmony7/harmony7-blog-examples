# Image Optimizer: transforming an image, with a fallback

Sends a request for `http-me.fastly.dev`'s sample JPEG (`/image-jpeg`) through `send-to-image-optimizer`, asking for it at 200 pixels wide in WebP, in the `us_east` region. If Image Optimizer refuses, it rebuilds the request, since the failed call consumed it, and serves the original image instead. The `x-image-optimizer` response header says which happened.

Full article: [Image Optimizer: The Request You Don't Get Back](https://behindthepanic.dev/posts/2026-09-25-image-optimizer/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

Then in another terminal:

```sh
curl -s -D - -o image.jpg http://127.0.0.1:7676/
```

Locally the transform never happens. Viceroy answers every Image Optimizer call with `unsupported`, so the header reports the error and `image.jpg` is the untransformed original. The `origin` backend is defined under `[local_server]` in `fastly.toml`, so there's nothing else to set up.

On a real service, Image Optimizer is a Beta that Fastly support enables for the service. It isn't something `fastly.toml` or `fastly products` can turn on for a Compute service, and every request through it is a billable image request.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
