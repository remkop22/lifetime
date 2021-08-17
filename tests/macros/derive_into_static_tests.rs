use lifetime::IntoStatic;
use std::{borrow::Cow, panic::Location};

#[test]
fn primitive_and_cow_str_struct() {
    #[derive(IntoStatic, Debug, Default, PartialEq, Clone)]
    struct Example<'a> {
        primitive: usize,
        cow: Cow<'a, str>,
    }

    fn into_static<'a>(x: Example<'a>) -> Example<'static> {
        IntoStatic::into_static(x)
    }

    let example = Example::default();
    let borrowed: Example = into_static(example.clone());
    assert_eq!(example, borrowed);
}

#[test]
fn two_generic_lifetime_params() {
    #[derive(IntoStatic, Debug, Default, PartialEq, Clone)]
    struct Example<'a, 'b> {
        a: Cow<'a, str>,
        b: Cow<'b, str>,
    }

    fn into_static<'a, 'b>(x: Example<'a, 'b>) -> Example<'static, 'static> {
        x.into_static()
    }

    let example = Example::default();
    let borrowed = into_static(example.clone());
    assert_eq!(example, borrowed);
}

#[test]
fn tuple_struct() {
    #[derive(IntoStatic, Debug, Default, PartialEq, Clone)]
    struct Example<'a>(usize, Cow<'a, str>);

    fn into_static<'a>(x: Example<'a>) -> Example<'static> {
        x.into_static()
    }

    let example = Example::default();
    let borrowed = into_static(example.clone());
    assert_eq!(example, borrowed);
}

#[test]
fn enum_() {
    #[allow(dead_code)]
    #[derive(IntoStatic, Debug, PartialEq, Clone)]
    enum Example<'a> {
        Primitive0 { number: usize },
        Primitive1(usize),
        Cow0 { string: Cow<'a, str> },
        Cow1(Cow<'a, str>),
    }

    fn into_static<'a>(x: Example<'a>) -> Example<'static> {
        IntoStatic::into_static(x)
    }

    let example = Example::Cow1(Default::default());
    let borrowed = into_static(example.clone());
    assert_eq!(example, borrowed);
}

#[test]
fn derive_struct_with_static_reference() {
    #[derive(IntoStatic, Debug, PartialEq, Clone)]
    struct Example<'a>(&'static Location<'static>, Cow<'a, str>);

    fn into_static<'a>(x: Example<'a>) -> Example<'static> {
        IntoStatic::into_static(x)
    }

    let example = Example(Location::caller(), Default::default());
    let borrowed = into_static(example.clone());
    assert_eq!(example, borrowed);
}
