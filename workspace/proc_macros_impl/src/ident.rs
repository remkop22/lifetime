use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

pub(crate) fn tuple_field_ident(index: usize) -> Ident {
    Ident::new(&format!("x{}", index), Span::mixed_site())
}

pub(crate) struct EnumVariantIdent {
    pub enum_ident: Ident,
    pub variant_ident: Ident,
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
