# ACLs: looking up an IP address

Opens the ACL linked as `blocklist` and looks up the client's IP address in it, or the address given in an `x-test-ip` header. It answers `403` when the match's action is `BLOCK`, and prints the raw JSON match document either way. It also tries to open an ACL that doesn't exist, to show `not-found`.

Full article: [ACLs: A Yes-or-No Question With a JSON Answer](https://behindthepanic.dev/posts/2026-09-28-acls/)

## Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- The [Fastly CLI](https://github.com/fastly/cli) (`brew install fastly/tap/fastly`), which bundles Viceroy for local serving

## Run it

```sh
fastly compute serve
```

`[local_server.acls]` in `fastly.toml` loads the `blocklist` ACL from `acl-entries.json`, so there's nothing else to set up. Then in another terminal:

```sh
# Your own address, 127.0.0.1, matches the blocked /32 rather than the allowed /8
curl -i http://127.0.0.1:7676/

# A neighbor that only matches the /8
curl -H "x-test-ip: 127.0.0.2" http://127.0.0.1:7676/

# No match at all
curl -H "x-test-ip: 198.51.100.7" http://127.0.0.1:7676/
```

On a real service, create the ACL with `fastly compute acl create`, add the entries with `fastly compute acl update --file=acl-entries.json`, link it to the service as `blocklist` with `fastly service resource-link create`, and activate the version. The name the program opens is the link's name.

## Why `wit/deps` is checked in, and why the Rust version is pinned

See the [`2026-08-03-hello-world-http-incoming` README](../2026-08-03-hello-world-http-incoming/README.md) — both apply here unchanged.
