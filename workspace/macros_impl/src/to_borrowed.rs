use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::convert::TryFrom;
use syn::{
    punctuated::Punctuated, token::Comma, Data, DataEnum, DeriveInput, Field, Fields,
    GenericArgument, GenericParam, Generics, Ident, Index, Lifetime, LifetimeDef, PathArguments,
    Type, TypeParamBound, TypePath, Variant,
};

pub fn derive(input: DeriveInput) -> TokenStream {
    let ref_lifetime = Lifetime::new("'ref_", Span::mixed_site());
    let generics = input.generics;
    let all_generics = add_lifetime(generics.clone(), ref_lifetime.clone());
    let borrowed_generics = replace_lifetimes(generics.clone(), ref_lifetime.clone());
    let ident = input.ident;
    let fn_body = match &input.data {
        Data::Struct(struct_data) => struct_constructor_call(&ident, &struct_data.fields),
        Data::Enum(enum_data) => matched_enum_constructor_call(&ident, enum_data),
        Data::Union(_) => panic!("only structs and enums are supported"),
    };
    quote! {
        impl #all_generics lifetime::ToBorrowed for & #ref_lifetime #ident #generics {
            type Borrowed = #ident #borrowed_generics;

            fn to_borrowed(self) -> #ident #borrowed_generics {
                use lifetime::ToBorrowed;

                #fn_body
            }
        }
    }
}

fn add_lifetime(mut generics: Generics, lifetime: Lifetime) -> Generics {
    generics
        .params
        .insert(0, GenericParam::Lifetime(LifetimeDef::new(lifetime)));
    generics
}

fn replace_lifetimes(mut generics: Generics, new: Lifetime) -> Generics {
    for old in generics.lifetimes_mut() {
        *old = LifetimeDef::new(new.clone());
    }
    generics
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
                    #ident: self.#ident.to_borrowed(),
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
                    self.#index.to_borrowed(),
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
    match ty {
        Type::Array(array) => type_has_generic_lifetime(&array.elem),
        Type::BareFn(_) => false,
        Type::Group(group) => type_has_generic_lifetime(&group.elem),
        Type::ImplTrait(impl_trait) => impl_trait.bounds.iter().any(|bound| match bound {
            TypeParamBound::Trait(_) => false,
            TypeParamBound::Lifetime(_) => true,
        }),
        Type::Infer(_) => true,
        Type::Macro(_) => true,
        Type::Never(_) => false,
        Type::Paren(paren) => type_has_generic_lifetime(&paren.elem),
        Type::Path(type_path) => type_path_has_generic_lifetime(type_path),
        Type::Ptr(_) => true,
        Type::Reference(_) => true,
        Type::Slice(slice) => type_has_generic_lifetime(&slice.elem),
        Type::TraitObject(trait_object) => trait_object.bounds.iter().any(|bound| match bound {
            TypeParamBound::Trait(_) => false,
            TypeParamBound::Lifetime(_) => true,
        }),
        Type::Tuple(tuple) => tuple.elems.iter().any(|ty| type_has_generic_lifetime(ty)),
        Type::Verbatim(_) => false,
        _ => false,
    }
}

fn type_path_has_generic_lifetime(type_path: &TypePath) -> bool {
    if let Some(_qself) = &type_path.qself {
        return true;
    }
    let last_segment = match type_path.path.segments.last() {
        None => return false,
        Some(l) => l,
    };
    match &last_segment.arguments {
        PathArguments::None => false,
        PathArguments::AngleBracketed(a) => a.args.iter().any(|arg| match arg {
            GenericArgument::Lifetime(_) => true,
            GenericArgument::Type(ty) => type_has_generic_lifetime(ty),
            GenericArgument::Binding(binding) => type_has_generic_lifetime(&binding.ty),
            GenericArgument::Constraint(_) => false,
            GenericArgument::Const(_) => false,
        }),
        PathArguments::Parenthesized(_) => false,
    }
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
                    #ident: #ident.to_borrowed(),
                }
            } else {
                quote! {
                    #ident: *#ident,
                }
            }
        }
        None => {
            let tuple_field_ident = tuple_field_ident(index);
            if type_has_generic_lifetime(&field.ty) {
                quote! {
                    #tuple_field_ident .to_borrowed(),
                }
            } else {
                quote! {
                    *#tuple_field_ident,
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
            impl<'ref_, 'a> lifetime::ToBorrowed for &'ref_ Example<'a> {
                type Borrowed = Example<'ref_>;

                fn to_borrowed(self) -> Example<'ref_> {
                    use lifetime::ToBorrowed;

                    Example {
                        primitive: self.primitive,
                        cow: self.cow.to_borrowed(),
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
            impl<'ref_, 'a> lifetime::ToBorrowed for &'ref_ Example<'a> {
                type Borrowed = Example<'ref_>;

                fn to_borrowed(self) -> Example<'ref_> {
                    use lifetime::ToBorrowed;

                    Example(self.0, self.1.to_borrowed(),)
                }
            }
        };
        test_derive_input_to_output(input, expected);
    }

    #[test]
    fn derive_enum() {
        let input = quote! {
            enum Example<'a> {
                Primitive0 {
                    number: usize
                },
                Primitive1(usize),
                Cow0 {
                    string: Cow<'a, str>
                },
                Cow1(Cow<'a, str>),
            }
        };
        let expected = quote! {
            impl<'ref_, 'a> lifetime::ToBorrowed for &'ref_ Example<'a> {
                type Borrowed = Example<'ref_>;

                fn to_borrowed(self) -> Example<'ref_> {
                    use lifetime::ToBorrowed;

                    match self {
                        Example::Primitive0 { number, } => Example::Primitive0 { number: *number, },
                        Example::Primitive1(x0,) => Example::Primitive1(*x0,),
                        Example::Cow0 { string, } => Example::Cow0 { string: string.to_borrowed(), },
                        Example::Cow1(x0,) => Example::Cow1(x0.to_borrowed(),),
                    }
                }
            }
        };
        test_derive_input_to_output(input, expected);
    }
}
