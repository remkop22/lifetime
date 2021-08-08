mod into_static;
mod to_borrowed;

pub use into_static::IntoStatic;
pub use to_borrowed::ToBorrowed;

#[cfg(feature = "macros")]
pub use macros::ToBorrowed;
