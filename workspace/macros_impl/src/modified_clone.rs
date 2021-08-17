use crate::ident::{tuple_field_ident, EnumVariantIdent};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, Data, DataEnum, Field, Fields, Ident, Variant};

pub(crate) struct ModifiedClone<'a> {
    pub ident: &'a Ident,
    pub data: &'a Data,
    pub struct_field_init: &'a FieldInit,
    pub enum_field_init: &'a FieldInit,
}

type FieldInit = dyn Fn(usize, &Field) -> TokenStream;

impl<'a> ModifiedClone<'a> {
    pub(crate) fn expression(self) -> TokenStream {
        match self.data {
            Data::Struct(struct_data) => {
                struct_constructor_call(self.ident, &struct_data.fields, self.struct_field_init)
            }
            Data::Enum(enum_data) => {
                matched_enum_constructor_call(self.ident, enum_data, self.enum_field_init)
            }
            Data::Union(_) => panic!("only structs and enums are supported"),
        }
    }
}

fn struct_constructor_call(
    ident: &Ident,
    fields: &Fields,
    struct_field_init: &FieldInit,
) -> TokenStream {
    match fields {
        Fields::Named(named_fields) => {
            let fields_initialization =
                struct_fields_initialization(&named_fields.named, struct_field_init);
            quote! {
                #ident { #fields_initialization }
            }
        }
        Fields::Unnamed(unnamed_fields) => {
            let fields_initialization =
                struct_fields_initialization(&unnamed_fields.unnamed, struct_field_init);
            quote! {
                #ident(#fields_initialization)
            }
        }
        Fields::Unit => panic!("unit structs are not supported"),
    }
}

fn struct_fields_initialization(
    fields: &Punctuated<Field, Comma>,
    struct_field_init: &FieldInit,
) -> TokenStream {
    fields
        .iter()
        .enumerate()
        .map(|(index, field)| struct_field_init(index, field))
        .collect()
}

fn matched_enum_constructor_call(
    enum_ident: &Ident,
    enum_data: &DataEnum,
    enum_field_init: &FieldInit,
) -> TokenStream {
    let patterns_and_construction: TokenStream = enum_data
        .variants
        .iter()
        .map(|variant| variant_pattern_and_construction(enum_ident, variant, enum_field_init))
        .collect();
    quote! {
        match self {
            #patterns_and_construction
        }
    }
}

fn variant_pattern_and_construction(
    enum_ident: &Ident,
    variant: &Variant,
    enum_field_init: &FieldInit,
) -> TokenStream {
    let ident = EnumVariantIdent {
        enum_ident: enum_ident.clone(),
        variant_ident: variant.ident.clone(),
    };
    match &variant.fields {
        Fields::Named(f) => {
            let enum_fields_pattern = enum_fields_pattern(&f.named);
            let enum_fields_initialization = enum_fields_initialization(&f.named, enum_field_init);
            quote! {
                #ident { #enum_fields_pattern } => #ident { #enum_fields_initialization },
            }
        }
        Fields::Unnamed(f) => {
            let enum_fields_pattern = enum_fields_pattern(&f.unnamed);
            let enum_fields_initialization =
                enum_fields_initialization(&f.unnamed, enum_field_init);
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

fn enum_fields_initialization(
    fields: &Punctuated<Field, Comma>,
    enum_field_init: &FieldInit,
) -> TokenStream {
    fields
        .iter()
        .enumerate()
        .map(|(index, field)| enum_field_init(index, field))
        .collect()
}
