use assert_matches::assert_matches;
use reuse::IntoStatic;
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
