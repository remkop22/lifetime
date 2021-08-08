use lifetime::ToBorrowed;
use std::borrow::Cow;

#[test]
fn primitive_and_cow_str_struct() {
    #[derive(ToBorrowed, Debug, Default, PartialEq)]
    struct Example<'a> {
        primitive: usize,
        cow: Cow<'a, str>,
    }

    let example = Example::default();
    let borrowed: Example = ToBorrowed::to_borrowed(&example);
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
