use crate::{
    generics::{assert_generics_are_supported, replace_lifetimes},
    ident::tuple_field_ident,
    modified_clone::ModifiedClone,
    type_::type_has_generic_lifetime,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::convert::TryFrom;
use syn::{DeriveInput, Field, Index, Lifetime};

pub fn derive(input: DeriveInput) -> TokenStream {
    let static_lifetime = Lifetime::new("'static", Span::mixed_site());
    let generics = input.generics;
    assert_generics_are_supported(&generics);
    let static_generics = replace_lifetimes(generics.clone(), &static_lifetime);
    let ident = input.ident;
    let fn_body = ModifiedClone {
        ident: &ident,
        data: &input.data,
        struct_field_init: &struct_field_initialization,
        enum_field_init: &enum_field_initialization,
    }
    .expression();
    quote! {
        impl #generics lifetime::IntoStatic for #ident #generics {
            type Static = #ident #static_generics;

            fn into_static(self) -> #ident #static_generics {
                use lifetime::IntoStatic;

                #fn_body
            }
        }
    }
}

fn struct_field_initialization(index: usize, field: &Field) -> TokenStream {
    match &field.ident {
        Some(ident) => {
            if type_has_generic_lifetime(&field.ty) {
                quote! {
                    #ident: self.#ident.into_static(),
                }
            } else {
                quote! {
                    #ident: self.#ident,
                }
            }
        }
        None => {
            let index = Index {
                index: u32::try_from(index).unwrap(),
                span: Span::mixed_site(),
            };
            if type_has_generic_lifetime(&field.ty) {
                quote! {
                    self.#index.into_static(),
                }
            } else {
                quote! {
                    self.#index,
                }
            }
        }
    }
}

fn enum_field_initialization(index: usize, field: &Field) -> TokenStream {
    match &field.ident {
        Some(ident) => {
            if type_has_generic_lifetime(&field.ty) {
                quote! {
                    #ident: #ident.into_static(),
                }
            } else {
                quote! {
                    #ident: #ident,
                }
            }
        }
        None => {
            let tuple_field_ident = tuple_field_ident(index);
            if type_has_generic_lifetime(&field.ty) {
                quote! {
                    #tuple_field_ident.into_static(),
                }
            } else {
                quote! {
                    #tuple_field_ident,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_derive_input_to_output(input: TokenStream, expected: TokenStream) {
        let actual = derive(parse(input));
        println!("{:#}", actual);
        assert_eq!(parse::<syn::Item>(actual), parse::<syn::Item>(expected),);
    }

    #[track_caller]
    fn parse<T: syn::parse::Parse>(tokens: TokenStream) -> T {
        syn::parse2(tokens).unwrap()
    }

    #[test]
    fn derive_primitive_and_cow_str_struct() {
        let input = quote! {
            struct Example<'a> {
                primitive: usize,
                cow: Cow<'a, str>,
            }
        };
        let expected = quote! {
            impl<'a> lifetime::IntoStatic for Example<'a> {
                type Static = Example<'static>;

                fn into_static(self) -> Example<'static> {
                    use lifetime::IntoStatic;

                    Example {
                        primitive: self.primitive,
                        cow: self.cow.into_static(),
                    }
                }
            }
        };
        test_derive_input_to_output(input, expected);
    }

    #[test]
    fn derive_tuple_struct() {
        let input = quote! {
            struct Example<'a>(usize, Cow<'a, str>);
        };
        let expected = quote! {
            impl<'a> lifetime::IntoStatic for Example<'a> {
                type Static = Example<'static>;

                fn into_static(self) -> Example<'static> {
                    use lifetime::IntoStatic;

                    Example(self.0, self.1.into_static(),)
                }
            }
        };
        test_derive_input_to_output(input, expected);
    }

    #[test]
    fn derive_enum() {
        let input = quote! {
            enum Example<'a> {
                Primitive0 { number: usize },
                Primitive1(usize),
                Cow0 { string: Cow<'a, str> },
                Cow1(Cow<'a, str>),
            }
        };
        let expected = quote! {
            impl<'a> lifetime::IntoStatic for Example<'a> {
                type Static = Example<'static>;

                fn into_static(self) -> Example<'static> {
                    use lifetime::IntoStatic;

                    match self {
                        Example::Primitive0 { number, } => Example::Primitive0 { number: number, },
                        Example::Primitive1(x0,) => Example::Primitive1(x0,),
                        Example::Cow0 { string, } => Example::Cow0 { string: string.into_static(), },
                        Example::Cow1(x0,) => Example::Cow1(x0.into_static(),),
                    }
                }
            }
        };
        test_derive_input_to_output(input, expected);
    }

    #[test]
    fn derive_struct_with_static_reference() {
        let input = quote! {
            struct Example<'a>(&'static Location<'static>, Cow<'a, str>);
        };
        let expected = quote! {
            impl<'a> lifetime::IntoStatic for Example<'a> {
                type Static = Example<'static>;

                fn into_static(self) -> Example<'static> {
                    use lifetime::IntoStatic;

                    Example(self.0, self.1.into_static(),)
                }
            }
        };
        test_derive_input_to_output(input, expected);
    }

    #[test]
    #[should_panic]
    fn derive_struct_with_generic_type() {
        let input = quote! {
            struct Example<T>(T);
        };
        derive(parse(input));
    }

    #[test]
    #[should_panic]
    fn derive_struct_with_generic_const() {
        let input = quote! {
            struct Example<const N: usize>;
        };
        derive(parse(input));
    }

    #[test]
    #[should_panic]
    fn derive_struct_with_lifetime_constrains() {
        let input = quote! {
            struct Example<'a, 'b: 'a>(Cow<'a, str>, Cow<'b, str>);
        };
        derive(parse(input));
    }
}
