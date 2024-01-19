# `accept-header`

An implementation of `headers::Header` for the `Accept` header as defined in [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110#name-accept).

Hopefully in the future this can be deprecated in favor of a canonical implentation in the `headers` library, but that appears not to be a current priority. See [hyperium/headers#53](https://github.com/hyperium/headers/issues/53).

## Including this in your Rust project

Because there already exists a different project with the same name on <https://crates.io>, this is not (yet) published there. Other methods

## Git Dependency

```toml
accept-header = { version = "0.1.0", git = "https://github.com/coriolinus/accept-header.git" }
```

## Cloudsmith Package Repository

[![Hosted By: Cloudsmith](https://img.shields.io/badge/OSS%20hosting%20by-cloudsmith-blue?logo=cloudsmith&style=for-the-badge)](https://cloudsmith.com)

Add the following lines to [a relevant `.cargo/config.toml`](https://doc.rust-lang.org/cargo/reference/config.html):

```toml
[registries.finvia-accept-header]
index = "sparse+https://cargo.cloudsmith.io/finvia/accept-header/"
```

Then depend on it as

```toml
accept-header = { version = "0.1.0", registry = "finvia-accept-header" }
```

Package repository hosting is graciously provided by  [Cloudsmith](https://cloudsmith.com).
Cloudsmith is the only fully hosted, cloud-native, universal package management solution, that
enables your organization to create, store and share packages in any format, to any place, with total
confidence.
