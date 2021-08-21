# lifetime

This crate provides traits and derive macros to change the lifetime of a type,
allowing efficient reuse of your stucts or enums with any lifetime requirement.

Have a look at the [documentation](https://docs.rs/lifetime/) for more information.

## Install

```toml
[dependencies]
lifetime = { version = "x.y", features = ["macros"] }
```

## Safety

This crate uses `#![forbid(unsafe_code)]`.
We want to keep this crate 100% safe and its dependencies to a minimum.
Currently this crate has no dependencies.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE)
or [MIT license](LICENSE-MIT)
at your option.

### Contribution

Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in this crate by you,
as defined in the Apache-2.0 license,
shall be dual licensed as above,
without any additional terms or conditions.
