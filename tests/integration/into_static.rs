use assert_matches::assert_matches;
use lifetime::IntoStatic;
use std::borrow::Cow;

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
