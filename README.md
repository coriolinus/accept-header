# `accept-header`

An implementation of `headers::Header` for the `Accept` header as defined in [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110#name-accept).

Hopefully in the future this can be deprecated in favor of a canonical implentation in the `headers` library, but that appears not to be a current priority. See [hyperium/headers#53](https://github.com/hyperium/headers/issues/53).

## Including this in your Rust project

Because there already exists a different project with the same name on <https://crates.io>, this is not (yet) published there. Instead, add a git dependency to your project:

```toml
accept-header = { version = "0.1.0", git = "https://github.com/coriolinus/accept-header.git" }
```
