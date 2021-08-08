use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated, token::Comma, Data, DataStruct, DeriveInput, Field, Fields,
    GenericArgument, GenericParam, Generics, Ident, Lifetime, LifetimeDef, PathArguments, Type,
    TypeParamBound, TypePath,
};

pub fn derive(input: DeriveInput) -> TokenStream {
    let ref_lifetime = Lifetime::new("'ref_", Span::mixed_site());
    let generics = input.generics;
    let all_generics = add_lifetime(generics.clone(), ref_lifetime.clone());
    let borrowed_generics = replace_lifetimes(generics.clone(), ref_lifetime.clone());
    let identifier = input.ident;
    let struct_data = match &input.data {
        Data::Struct(s) => s,
        _ => panic!("only structs are supported"),
    };
    let fn_body = constructor_call(&identifier, struct_data);
    quote! {
        impl #all_generics lifetime::ToBorrowed for & #ref_lifetime #identifier #generics {
            type Borrowed = #identifier #borrowed_generics;

            fn to_borrowed(self) -> #identifier #borrowed_generics {
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

fn constructor_call(ident: &Ident, struct_data: &DataStruct) -> TokenStream {
    match &struct_data.fields {
        Fields::Named(named_fields) => {
            let fields_initialization = fields_initialization(&named_fields.named);
            quote! {
                #ident { #fields_initialization }
            }
        }
        Fields::Unnamed(unnamed_fields) => {
            let fields_initialization = fields_initialization(&unnamed_fields.unnamed);
            quote! {
                #ident(#fields_initialization)
            }
        }
        Fields::Unit => panic!("unit structs are not supported"),
    }
}

fn fields_initialization(fields: &Punctuated<Field, Comma>) -> TokenStream {
    fields
        .iter()
        .enumerate()
        .map(|(index, field)| field_initialization(index, field))
        .collect()
}

fn field_initialization(index: usize, field: &Field) -> TokenStream {
    match &field.ident {
        Some(identifier) => {
            if type_has_generic_lifetime(&field.ty) {
                quote! {
                    #identifier: self.#identifier.to_borrowed(),
                }
            } else {
                quote! {
                    #identifier: self.#identifier,
                }
            }
        }
        None => {
            if type_has_generic_lifetime(&field.ty) {
                quote! {
                    self.#index,
                }
            } else {
                quote! {
                    self.#index.to_borrowed(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn parse<T: syn::parse::Parse>(tokens: TokenStream) -> T {
        syn::parse2(tokens).unwrap()
    }

    #[test]
    fn primitive_and_cow_str_struct() {
        let input = quote! {
            struct Example<'a> {
                primitive: usize,
                cow: Cow<'a, str>,
            }
        };
        let actual = derive(parse(input));
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
        println!("{:#}", actual);
        assert_eq!(parse::<syn::Item>(actual), parse::<syn::Item>(expected),);
    }

    #[test]
    #[should_panic]
    fn enum_() {
        let input = quote! {
            enum Example<'a> {
                Primitive {
                    number: usize
                },
                Cow(Cow<'a, str>),
            }
        };
        derive(parse(input));
    }
}
