//! Derive macro for [`enum_map_lite::Enum`].
//!
//! Generates `into_usize` (declaration-order indexing) and the `EnumArray<V>`
//! backing-array impl. Keys purely on the **discriminant**: one slot per
//! variant, field contents ignored. Works on any enum regardless of its
//! variants' fields. Intentionally does **not** generate `from_usize`, so there
//! is no key/pair iteration.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Enum)]
pub fn derive_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Enum(data) = &input.data else {
        return syn::Error::new_spanned(&input, "`Enum` can only be derived for enums")
            .to_compile_error()
            .into();
    };

    let n = data.variants.len();
    // One slot per variant; ignore any fields with a wildcard pattern.
    let arms = data.variants.iter().enumerate().map(|(i, variant)| {
        let ident = &variant.ident;
        let pattern = match &variant.fields {
            Fields::Unit => quote! { #name::#ident },
            Fields::Unnamed(_) => quote! { #name::#ident(..) },
            Fields::Named(_) => quote! { #name::#ident { .. } },
        };
        quote! { #pattern => #i, }
    });

    quote! {
        impl ::enum_map_lite::Enum for #name {
            const LENGTH: usize = #n;

            fn into_usize(self) -> usize {
                match self {
                    #(#arms)*
                }
            }
        }

        impl<V> ::enum_map_lite::EnumArray<V> for #name {
            type Array = [V; #n];

            fn from_index_fn<F: ::core::ops::FnMut(usize) -> V>(f: F) -> [V; #n] {
                ::core::array::from_fn(f)
            }
        }
    }
    .into()
}
