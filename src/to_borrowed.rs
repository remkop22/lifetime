#[cfg(feature = "alloc")]
use alloc::borrow::{Borrow, Cow, ToOwned};

/// A trait for downgrading the lifetime of a type.
///
#[cfg_attr(
    feature = "alloc",
    doc = r##"
# Examples
```rust
use lifetime::ToBorrowed;
use std::borrow::Cow;

let owned: Cow<'static, str> = Cow::Owned(String::from("Hi"));
let mut borrowed: Cow<'_, str> = owned.to_borrowed();

assert_eq!(borrowed, "Hi");
assert_matches::assert_matches!(borrowed, Cow::Borrowed(_));

borrowed = Cow::Borrowed("Bye");
assert_eq!(owned, "Hi");
```
"##
)]
pub trait ToBorrowed {
    type Borrowed;

    fn to_borrowed(self) -> Self::Borrowed;
}

#[cfg(feature = "alloc")]
impl<'b, 'c, B> ToBorrowed for &'c Cow<'b, B>
where
    B: ToOwned + ?Sized + 'static,
{
    type Borrowed = Cow<'c, B>;

    #[inline]
    fn to_borrowed(self) -> Cow<'c, B> {
        Cow::Borrowed(self.borrow())
    }
}

impl<'o, T> ToBorrowed for &'o Option<T>
where
    for<'t> &'t T: ToBorrowed,
{
    type Borrowed = Option<<&'o T as ToBorrowed>::Borrowed>;

    #[inline]
    fn to_borrowed(self) -> Option<<&'o T as ToBorrowed>::Borrowed> {
        self.as_ref().map(ToBorrowed::to_borrowed)
    }
}

#[cfg(feature = "alloc")]
#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn option_some_cow() {
        let some_borrowed = Some(Cow::Borrowed("Elm"));
        let actual = some_borrowed.to_borrowed();
        assert_eq!(actual, some_borrowed);
        assert_matches!(actual, Some(Cow::Borrowed(_)));
    }

    #[test]
    fn option_none_cow() {
        let option: Option<Cow<str>> = None;
        let actual = option.to_borrowed();
        assert_eq!(actual, None);
    }
}
