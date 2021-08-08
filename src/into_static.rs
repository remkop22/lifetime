use std::borrow::Cow;

/// A trait for upgrading the lifetime of a type.
///
/// # Examples
/// ```
/// use lifetime::IntoStatic;
/// use std::borrow::Cow;
///
/// let string = String::from("Hi");
/// let borrowed: Cow<'_, str> = Cow::Borrowed(&*string);
/// let static_string: Cow<'static, str> = borrowed.into_static();
///
/// // we can still use static_string after this drop
/// drop(string);
///
/// assert_eq!(static_string, "Hi");
/// assert_matches::assert_matches!(static_string, Cow::Owned(_));
/// ```
pub trait IntoStatic {
    type Static: 'static;

    fn into_static(self) -> Self::Static;
}

impl<'b, B> IntoStatic for Cow<'b, B>
where
    B: ToOwned + ?Sized + 'static,
{
    type Static = Cow<'static, B>;

    #[inline]
    fn into_static(self) -> Cow<'static, B> {
        Cow::Owned(self.into_owned())
    }
}

impl<T> IntoStatic for Option<T>
where
    T: IntoStatic,
{
    type Static = Option<T::Static>;

    #[inline]
    fn into_static(self) -> Option<T::Static> {
        self.map(IntoStatic::into_static)
    }
}

#[cfg(feature = "unstable")]
impl<T, E> IntoStatic for Result<T, E>
where
    T: IntoStatic,
    E: IntoStatic,
{
    type Static = Result<T::Static, E::Static>;

    #[inline]
    fn into_static(self) -> Result<T::Static, E::Static> {
        self.map(IntoStatic::into_static)
            .map_err(IntoStatic::into_static)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn option_some_cow() {
        let some_borrowed = Some(Cow::Borrowed("Elm"));
        let actual = some_borrowed.clone().into_static();
        assert_eq!(actual, some_borrowed);
        assert_matches!(actual, Some(Cow::Owned(_)));
    }

    #[test]
    fn option_none_cow() {
        let option: Option<Cow<str>> = None;
        let actual = option.clone().into_static();
        assert_eq!(actual, None);
    }

    #[cfg(feature = "unstable")]
    #[test]
    fn result_ok_cow() {
        let ok_borrowed: Result<Cow<str>, Cow<str>> = Ok(Cow::Borrowed("Elm"));
        let actual = ok_borrowed.clone().into_static();
        assert_eq!(actual, ok_borrowed);
        assert_matches!(actual, Ok(Cow::Owned(_)));
    }

    #[cfg(feature = "unstable")]
    #[test]
    fn result_err_cow() {
        let ok_borrowed: Result<Cow<str>, Cow<str>> = Err(Cow::Borrowed("Elm"));
        let actual = ok_borrowed.clone().into_static();
        assert_eq!(actual, ok_borrowed);
        assert_matches!(actual, Err(Cow::Owned(_)));
    }
}
