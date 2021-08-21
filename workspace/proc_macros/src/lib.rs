#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro generating an impl of the trait `IntoStatic`.
#[proc_macro_derive(IntoStatic)]
pub fn derive_into_static(input: TokenStream) -> TokenStream {
    lifetime_proc_macros_impl::into_static::derive(parse_macro_input!(input as DeriveInput)).into()
}

/// Derive macro generating an impl of the trait `ToBorrowed`.
#[proc_macro_derive(ToBorrowed)]
pub fn derive_to_borrowed(input: TokenStream) -> TokenStream {
    lifetime_proc_macros_impl::to_borrowed::derive(parse_macro_input!(input as DeriveInput)).into()
}
