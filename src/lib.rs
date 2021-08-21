/*!
This crate provides traits and derive macros to change the lifetime of a type,
allowing efficient reuse of your stucts or enums with any lifetime requirement.

# `macros` feature

This crate's macros are not enabled by default.
Use the following in your `Cargo.toml`,
replacing `x.y` with the version you want,
to enable macros.
```toml
[dependencies]
lifetime = { version = "x.y", features = ["macros"] }
```

# no_std

Use the following in your `Cargo.toml`,
replacing `x.y` with the version you want,
to disable the default `std` and `alloc` features.
```toml
[dependencies]
lifetime = { version = "x.y", default-features = false }
```

*/
#![cfg_attr(
    feature = "alloc",
    doc = r##"

# Examples

Note that the examples use explicit types and lifetimes to be more illustrative.
Typically the lifetimes and/or types can be
[inferred](https://doc.rust-lang.org/rust-by-example/types/inference.html)
by the compiler.

```rust
use lifetime::{IntoStatic, ToBorrowed};
use std::borrow::Cow;

let alice: Cow<'static, str> = Cow::Borrowed("Alice");

let borrowed_alice: Cow<'_, str> = alice.to_borrowed();
assert_eq!(borrowed_alice, alice);

let static_alice: Cow<'static, str> = borrowed_alice.into_static();
assert_eq!(static_alice, alice);
```

"##
)]
#![cfg_attr(
    all(feature = "macros", feature = "alloc"),
    doc = r##"
Here's an example with a struct using the derive macros.
The `macros` feature needs to be enabled.
```rust
use lifetime::{IntoStatic, ToBorrowed};
use std::borrow::Cow;

#[derive(IntoStatic, ToBorrowed)]
struct Header<'a> {
    name: Cow<'a, str>,
    value: Cow<'a, str>,
}

let xml: Header<'static> = Header {
    name: "content".into(),
    value: "xml".into(),
};
let mut xml_or_json: Header<'_> = xml.to_borrowed();
xml_or_json.value += " or json"; // creates a new String

assert_eq!(xml.value, "xml"); // xml header has not moved
assert_eq!(xml_or_json.value, "xml or json");

// IntoStatic converts all borrowed `&str` into `String`
let static_xml_or_json: Header<'static> = xml_or_json.into_static();
assert_eq!(static_xml_or_json.value, "xml or json");
```
"##
)]
#![forbid(unsafe_code)]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod into_static;
mod to_borrowed;

pub use into_static::IntoStatic;
pub use to_borrowed::ToBorrowed;

#[cfg(feature = "macros")]
pub use macros::{IntoStatic, ToBorrowed};
