use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(IntoStatic)]
pub fn derive_into_static(input: TokenStream) -> TokenStream {
    macros_impl::into_static::derive(parse_macro_input!(input as DeriveInput)).into()
}

#[proc_macro_derive(ToBorrowed)]
pub fn derive_to_borrowed(input: TokenStream) -> TokenStream {
    macros_impl::to_borrowed::derive(parse_macro_input!(input as DeriveInput)).into()
}
