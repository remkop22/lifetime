use lifetime::ToBorrowed;
use std::{borrow::Cow, panic::Location};

#[test]
fn primitive_and_cow_str_struct() {
    #[derive(ToBorrowed, Debug, Default, PartialEq)]
    struct Example<'a> {
        primitive: usize,
        cow: Cow<'a, str>,
    }

    fn to_borrowed<'r, 'a>(x: &'r Example<'a>) -> Example<'r> {
        ToBorrowed::to_borrowed(x)
    }

    let example = Example::default();
    let borrowed: Example = to_borrowed(&example);
    assert_eq!(example, borrowed);
}

#[test]
fn two_generic_lifetime_params() {
    #[derive(ToBorrowed, Debug, Default, PartialEq)]
    struct Example<'a, 'b> {
        a: Cow<'a, str>,
        b: Cow<'b, str>,
    }

    fn to_borrowed<'r, 'a, 'b>(x: &'r Example<'a, 'b>) -> Example<'r, 'r> {
        x.to_borrowed()
    }

    let example = Example::default();
    let borrowed = to_borrowed(&example);
    assert_eq!(example, borrowed);
}

#[test]
fn tuple_struct() {
    #[derive(ToBorrowed, Debug, Default, PartialEq)]
    struct Example<'a>(usize, Cow<'a, str>);

    fn to_borrowed<'r, 'a>(x: &'r Example<'a>) -> Example<'r> {
        x.to_borrowed()
    }

    let example = Example::default();
    let borrowed = to_borrowed(&example);
    assert_eq!(example, borrowed);
}

#[test]
fn enum_() {
    #[allow(dead_code)]
    #[derive(ToBorrowed, Debug, PartialEq)]
    enum Example<'a> {
        Primitive0 { number: usize },
        Primitive1(usize),
        Cow0 { string: Cow<'a, str> },
        Cow1(Cow<'a, str>),
    }

    fn to_borrowed<'r, 'a>(x: &'r Example<'a>) -> Example<'r> {
        ToBorrowed::to_borrowed(x)
    }

    let example = Example::Cow1(Default::default());
    let borrowed = to_borrowed(&example);
    assert_eq!(example, borrowed);
}

#[test]
fn derive_struct_with_static_reference() {
    #[derive(ToBorrowed, Debug, PartialEq)]
    struct Example<'a>(&'static Location<'static>, Cow<'a, str>);

    fn to_borrowed<'r, 'a>(x: &'r Example<'a>) -> Example<'r> {
        ToBorrowed::to_borrowed(x)
    }

    let example = Example(Location::caller(), Default::default());
    let borrowed = to_borrowed(&example);
    assert_eq!(example, borrowed);
}
