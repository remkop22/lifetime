use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::convert::TryFrom;
use syn::{
    punctuated::Punctuated,
    token::Comma,
    visit::{self, Visit},
    Data, DataEnum, DeriveInput, Field, Fields, Ident, Index, Lifetime, Type, Variant,
};

use crate::generics::{has_generic_type, replace_lifetimes};

pub fn derive(input: DeriveInput) -> TokenStream {
    let static_lifetime = Lifetime::new("'static", Span::mixed_site());
    let generics = input.generics;
    if has_generic_type(&generics) {
        panic!("generic type parameters are not supported");
    }
    let static_generics = replace_lifetimes(generics.clone(), static_lifetime);
    let ident = input.ident;
    let fn_body = match &input.data {
        Data::Struct(struct_data) => struct_constructor_call(&ident, &struct_data.fields),
        Data::Enum(enum_data) => matched_enum_constructor_call(&ident, enum_data),
        Data::Union(_) => panic!("only structs and enums are supported"),
    };
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

fn struct_constructor_call(ident: &Ident, fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(named_fields) => {
            let fields_initialization = struct_fields_initialization(&named_fields.named);
            quote! {
                #ident { #fields_initialization }
            }
        }
        Fields::Unnamed(unnamed_fields) => {
            let fields_initialization = struct_fields_initialization(&unnamed_fields.unnamed);
            quote! {
                #ident(#fields_initialization)
            }
        }
        Fields::Unit => panic!("unit structs are not supported"),
    }
}

fn struct_fields_initialization(fields: &Punctuated<Field, Comma>) -> TokenStream {
    fields
        .iter()
        .enumerate()
        .map(|(index, field)| struct_field_initialization(index, field))
        .collect()
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

fn type_has_generic_lifetime(ty: &Type) -> bool {
    struct Visitor {
        has_generic_lifetime: bool,
    }
    impl Visit<'_> for Visitor {
        fn visit_lifetime(&mut self, lifetime: &Lifetime) {
            if lifetime.ident != "static" {
                self.has_generic_lifetime = true
            } else {
                visit::visit_lifetime(self, lifetime)
            }
        }
    }
    let mut visitor = Visitor {
        has_generic_lifetime: false,
    };
    Visit::visit_type(&mut visitor, ty);
    visitor.has_generic_lifetime
}

fn matched_enum_constructor_call(enum_ident: &Ident, enum_data: &DataEnum) -> TokenStream {
    let patterns_and_construction: TokenStream = enum_data
        .variants
        .iter()
        .map(|variant| variant_pattern_and_construction(enum_ident, variant))
        .collect();
    quote! {
        match self {
            #patterns_and_construction
        }
    }
}

fn variant_pattern_and_construction(enum_ident: &Ident, variant: &Variant) -> TokenStream {
    let ident = EnumVariantIdent {
        enum_ident: enum_ident.clone(),
        variant_ident: variant.ident.clone(),
    };
    match &variant.fields {
        Fields::Named(f) => {
            let enum_fields_pattern = enum_fields_pattern(&f.named);
            let enum_fields_initialization = enum_fields_initialization(&f.named);
            quote! {
                #ident { #enum_fields_pattern } => #ident { #enum_fields_initialization },
            }
        }
        Fields::Unnamed(f) => {
            let enum_fields_pattern = enum_fields_pattern(&f.unnamed);
            let enum_fields_initialization = enum_fields_initialization(&f.unnamed);
            quote! {
                #ident ( #enum_fields_pattern ) => #ident ( #enum_fields_initialization ),
            }
        }
        Fields::Unit => todo!(),
    }
}

fn enum_fields_pattern(fields: &Punctuated<Field, Comma>) -> TokenStream {
    fields
        .iter()
        .enumerate()
        .map(|(i, f)| enum_field_pattern(i, f))
        .collect()
}

fn enum_field_pattern(index: usize, field: &Field) -> TokenStream {
    match &field.ident {
        Some(ident) => quote! {
            #ident,
        },
        None => {
            let tuple_field_ident = tuple_field_ident(index);
            quote! {
                #tuple_field_ident,
            }
        }
    }
}

fn tuple_field_ident(index: usize) -> Ident {
    Ident::new(&format!("x{}", index), Span::mixed_site())
}

fn enum_fields_initialization(fields: &Punctuated<Field, Comma>) -> TokenStream {
    fields
        .iter()
        .enumerate()
        .map(|(index, field)| enum_field_initialization(index, field))
        .collect()
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

struct EnumVariantIdent {
    enum_ident: Ident,
    variant_ident: Ident,
}

impl quote::ToTokens for EnumVariantIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            enum_ident,
            variant_ident,
        } = self;
        let ident = quote! { #enum_ident :: #variant_ident };
        tokens.extend(std::iter::once(ident));
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
}
