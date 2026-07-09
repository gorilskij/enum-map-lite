//! Proc-macros for [`enum-map-lite`](https://docs.rs/enum-map-lite):
//! `#[derive(Enum)]` and the function-like `enum_map!`.
//!
//! `#[derive(Enum)]` keys purely on the **discriminant** — one slot per variant,
//! fields ignored — and works on any enum (including generic and data-carrying
//! ones). It generates:
//! - `into_usize` (declaration-order index) for value lookups, and
//! - a hidden fieldless "shadow" enum mirroring the variant tags, exposed only
//!   through the `Enum::Shadow` associated type. `enum_map!` uses the shadow to
//!   turn a *pattern* key (e.g. `E::V { .. }`) into an index without ever
//!   constructing a value of `E`.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DeriveInput, Expr, Fields, Ident, Pat, PatStruct, PatTupleStruct,
    Path, Token,
};

// ---------------------------------------------------------------------------
// #[derive(Enum)]
// ---------------------------------------------------------------------------

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
    let variant_idents: Vec<&Ident> = data.variants.iter().map(|v| &v.ident).collect();

    // One slot per variant; ignore any fields with a wildcard pattern.
    let into_usize_arms = data.variants.iter().enumerate().map(|(i, variant)| {
        let ident = &variant.ident;
        let pattern = match &variant.fields {
            Fields::Unit => quote! { Self::#ident },
            Fields::Unnamed(_) => quote! { Self::#ident(..) },
            Fields::Named(_) => quote! { Self::#ident { .. } },
        };
        quote! { #pattern => #i, }
    });

    // Fieldless mirror of the variant tags. `V as usize` yields the declaration
    // index (0, 1, 2, ...), matching `into_usize`. Non-generic on purpose — the
    // tags don't depend on the enum's type parameters — so it can be named
    // without spelling out any generics.
    let shadow = format_ident!("__EnumMapLiteShadow_{}", name);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // The `EnumArray<V>` impl needs an extra `V` type parameter.
    let mut generics_v = input.generics.clone();
    generics_v.params.push(syn::parse_quote!(V));
    let (impl_generics_v, _, where_clause_v) = generics_v.split_for_impl();

    quote! {
        #[doc(hidden)]
        #[repr(usize)]
        #[derive(::core::marker::Copy, ::core::clone::Clone)]
        #[allow(non_camel_case_types, dead_code)]
        pub enum #shadow {
            #(#variant_idents,)*
        }

        impl #impl_generics ::enum_map_lite::Enum for #name #ty_generics #where_clause {
            const LENGTH: usize = #n;
            type Shadow = #shadow;

            fn into_usize(self) -> usize {
                match self {
                    #(#into_usize_arms)*
                }
            }
        }

        impl #impl_generics_v ::enum_map_lite::EnumArray<V> for #name #ty_generics #where_clause_v {
            type Array = [V; #n];

            fn from_index_fn<F: ::core::ops::FnMut(usize) -> V>(f: F) -> [V; #n] {
                ::core::array::from_fn(f)
            }
        }
    }
    .into()
}

// ---------------------------------------------------------------------------
// enum_map! { PATTERN => value, ..., _ => default }
// ---------------------------------------------------------------------------

struct Arm {
    pat: Pat,
    val: Expr,
}

impl Parse for Arm {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pat = input.call(Pat::parse_single)?;
        input.parse::<Token![=>]>()?;
        let val = input.parse()?;
        Ok(Arm { pat, val })
    }
}

struct EnumMapInput {
    arms: Punctuated<Arm, Token![,]>,
}

impl Parse for EnumMapInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(EnumMapInput {
            arms: Punctuated::parse_terminated(input)?,
        })
    }
}

#[proc_macro]
pub fn enum_map(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as EnumMapInput);
    match build_enum_map(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn build_enum_map(input: EnumMapInput) -> syn::Result<TokenStream2> {
    let mut default: Option<Expr> = None;
    // (owner path, variant ident, value)
    let mut arms: Vec<(Path, Ident, Expr)> = Vec::new();
    let mut owner: Option<Path> = None;

    for arm in input.arms {
        if matches!(arm.pat, Pat::Wild(_)) {
            if default.is_some() {
                return Err(syn::Error::new_spanned(
                    arm.pat,
                    "duplicate `_` catch-all arm",
                ));
            }
            default = Some(arm.val);
            continue;
        }

        let (arm_owner, variant) = parse_key(&arm.pat)?;

        match &owner {
            Some(existing)
                if existing.to_token_stream().to_string()
                    != arm_owner.to_token_stream().to_string() =>
            {
                return Err(syn::Error::new_spanned(
                    &arm_owner,
                    "all keys in an `enum_map!` must belong to the same enum",
                ));
            }
            None => owner = Some(arm_owner.clone()),
            _ => {}
        }

        arms.push((arm_owner, variant, arm.val));
    }

    // Pins the map's key type to the enum named in the patterns, and rejects a
    // generic owner (which can't be spelled here) with a clear compile error.
    let witness = owner.as_ref().map(|o| {
        quote! {
            if false {
                let _: &::enum_map_lite::EnumMap<#o, _> = &__map;
            }
        }
    });

    let index = |owner: &Path, variant: &Ident| {
        quote! { <#owner as ::enum_map_lite::Enum>::Shadow::#variant as usize }
    };

    Ok(match default {
        // With a `_` catch-all: fill every slot with the default, then overwrite.
        Some(def) => {
            let assigns = arms.iter().map(|(o, v, val)| {
                let idx = index(o, v);
                quote! { __map.as_mut_slice()[#idx] = #val; }
            });
            quote! {{
                let mut __map = ::enum_map_lite::EnumMap::from_index_fn(|_| #def);
                #witness
                #(#assigns)*
                __map
            }}
        }
        // Exhaustive, no catch-all: build an Option map, then unwrap (panics at
        // construction if a variant was left unset).
        None => {
            let assigns = arms.iter().map(|(o, v, val)| {
                let idx = index(o, v);
                quote! { __map.as_mut_slice()[#idx] = ::core::option::Option::Some(#val); }
            });
            quote! {{
                let mut __map =
                    ::enum_map_lite::EnumMap::from_index_fn(|_| ::core::option::Option::None);
                #witness
                #(#assigns)*
                __map.map(|__slot| {
                    __slot.expect(
                        "enum_map!: a variant was left unset (add a `_ => ...` catch-all)",
                    )
                })
            }}
        }
    })
}

/// Extracts `(owner path, variant ident)` from a key pattern, enforcing that
/// fields are only `_` or `..` (no bindings, no value matching).
fn parse_key(pat: &Pat) -> syn::Result<(Path, Ident)> {
    let path = match pat {
        Pat::Path(p) => &p.path,
        Pat::TupleStruct(t) => {
            validate_tuple(t)?;
            &t.path
        }
        Pat::Struct(s) => {
            validate_struct(s)?;
            &s.path
        }
        _ => {
            return Err(syn::Error::new_spanned(
                pat,
                "expected `Type::Variant`, `Type::Variant(..)`, or `Type::Variant { .. }`",
            ))
        }
    };
    split_variant(path)
}

fn validate_tuple(t: &PatTupleStruct) -> syn::Result<()> {
    for elem in &t.elems {
        if !matches!(elem, Pat::Wild(_) | Pat::Rest(_)) {
            return Err(syn::Error::new(
                elem.span(),
                "tuple-variant fields may only be `_` or `..` (no bindings)",
            ));
        }
    }
    Ok(())
}

fn validate_struct(s: &PatStruct) -> syn::Result<()> {
    for field in &s.fields {
        // Shorthand like `{ x }` binds `x`; require explicit `field: _`.
        if field.colon_token.is_none() || !matches!(&*field.pat, Pat::Wild(_)) {
            return Err(syn::Error::new(
                field.span(),
                "struct-variant fields may only be `field: _` (plus an optional `..`); no bindings",
            ));
        }
    }
    Ok(())
}

fn split_variant(path: &Path) -> syn::Result<(Path, Ident)> {
    if path.segments.len() < 2 {
        return Err(syn::Error::new_spanned(
            path,
            "expected a qualified `Type::Variant` path",
        ));
    }
    let variant = path.segments.last().unwrap().ident.clone();
    let mut owner = path.clone();
    owner.segments = owner.segments.into_pairs().collect::<Vec<_>>()[..path.segments.len() - 1]
        .iter()
        .cloned()
        .collect();
    // Drop any trailing path separator left on the final owner segment.
    if let Some(last) = owner.segments.pop() {
        owner.segments.push_value(last.into_value());
    }
    Ok((owner, variant))
}
