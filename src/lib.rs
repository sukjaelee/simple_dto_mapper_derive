//!
//! ## Detailed Documentation
//!
//! The `DtoFrom` derive macro generates a `From<Source>` implementation for the target struct.
//! It supports the following attributes on structs and fields:
//!
//! ### Struct-level Attribute (required)
//! - `#[dto(from = Type)]`
//!   - Specifies the source type `Type` (a Rust `Path`) from which to map.
//!   - Must appear on the same item as `#[derive(DtoFrom)]`.
//!
//! ### Field-level Attributes
//! - `#[dto(rename = "orig_name")]`
//!   - Maps the struct field to a differently named source field (by name).
//! - `#[dto(transform_fn = path::to::function)]`
//!   - Applies the function `path::to::function(source_field)` to transform the input.
//!   - The function must have the signature `FnOnce(SourceFieldType) -> FieldType`.
//! - `#[dto(skip)]`
//!   - Omits this field from the mapping; the field is initialized with `Default::default()`.
//! - `#[dto(into)]`
//!   - Uses `Into` to convert the source field into the DTO field type, i.e. `source_field.into()`.
//!
//! ### Usage Example
//!
//! ```rust
//! use simple_dto_mapper_derive::DtoFrom;
//!
//! mod types {
//!     #[derive(Debug)]
//!     pub struct User {
//!         pub id: String,
//!         pub name: String,
//!         pub age: u32,
//!         pub password: String, // intentionally not mapped into the DTO
//!         pub status: SourceStatus,
//!     }
//!
//!     #[derive(Debug, Clone)]
//!     pub enum SourceStatus { Active, Inactive }
//!
//!     #[derive(Debug, Clone)]
//!     pub enum DtoStatus { Active, Inactive }
//!
//!     // Infallible enum conversion for `#[dto(into)]`
//!     impl From<SourceStatus> for DtoStatus {
//!         fn from(s: SourceStatus) -> Self {
//!             match s {
//!                 SourceStatus::Active => DtoStatus::Active,
//!                 SourceStatus::Inactive => DtoStatus::Inactive,
//!             }
//!         }
//!     }
//!
//!     // Owned transform used by `transform_fn`
//!     pub fn mask_name(name: String) -> String {
//!         let ch = name.chars().next().unwrap_or('*');
//!         format!("{ch}***")
//!     }
//! }
//!
//! #[derive(DtoFrom, Debug)]
//! #[dto(from = types::User)]
//! struct UserDto {
//!     /// `rename` + `transform_fn`: read from `name` and apply `types::mask_name`
//!     #[dto(rename = "name", transform_fn = types::mask_name)]
//!     display_name: String,
//!
//!     /// Direct mapping
//!     age: u32,
//!
//!     /// `skip`: initialize with `Default::default()`
//!     #[dto(skip)]
//!     note: Option<String>,
//!
//!     /// `into`: uses `From<SourceStatus> for DtoStatus`
//!     #[dto(into)]
//!     status: types::DtoStatus,
//! }
//!
//! // Convert
//! let user = types::User {
//!     id: "u1".into(),
//!     name: "Alice".into(),
//!     age: 30,
//!     password: "secret".into(),
//!     status: types::SourceStatus::Active,
//! };
//! let dto: UserDto = user.into();
//! ```
//!
//! ### Additional Example: Collections
//!
//! Converting collections and options is handled explicitly with `transform_fn` helpers.
//! This keeps behavior clear without extra macro features.
//!
//! ```rust
//! use simple_dto_mapper_derive::DtoFrom;
//! use chrono::{DateTime, Utc};
//!
//! mod types {
//!     #[derive(Debug)]
//!     pub struct SourceTag(pub String);
//!     
//!     #[derive(Debug)]
//!     pub struct SourceAuthor { pub name: String }
//!
//!     #[derive(Debug)]
//!     pub struct Article {
//!         pub id: String,
//!         pub title: String,
//!         pub labels: Vec<SourceTag>,           // -> Vec<DtoTag>
//!         pub keywords: Vec<String>,            // same type
//!         pub author: Option<SourceAuthor>,      // -> Option<DtoAuthor>
//!         pub published_at: Option<chrono::DateTime<chrono::Utc>>, // -> Option<DateTime<Utc>>
//!     }
//!
//!     #[derive(Debug, Clone, PartialEq, Eq)]
//!     pub struct DtoTag(pub String);
//!     impl From<SourceTag> for DtoTag { fn from(t: SourceTag) -> Self { DtoTag(t.0) } }
//!
//!     #[derive(Debug, Clone)]
//!     pub struct DtoAuthor { pub name: String }
//!     impl From<SourceAuthor> for DtoAuthor { fn from(a: SourceAuthor) -> Self { DtoAuthor { name: a.name } } }
//!
//!     // Helpers for transform_fn
//!     pub fn vec_into<T, U>(v: Vec<T>) -> Vec<U> where U: From<T> {
//!         v.into_iter().map(Into::into).collect()
//!     }
//!     pub fn opt_into<T, U>(o: Option<T>) -> Option<U> where U: From<T> {
//!         o.map(Into::into)
//!     }
//! }
//!
//! #[derive(DtoFrom, Debug)]
//! #[dto(from = types::Article)]
//! struct ArticleDto {
//!     /// Direct mapping (same type)
//!     id: String,
//!     
//!     /// Rename + direct mapping
//!     #[dto(rename = "title")]
//!     headline: String,
//!
//!     /// Same type; no transform needed
//!     keywords: Vec<String>,
//!
//!     /// Vec<SourceTag> → Vec<DtoTag>
//!     #[dto(rename = "labels", transform_fn = types::vec_into::<types::SourceTag, types::DtoTag>)]
//!     tags: Vec<types::DtoTag>,
//!
//!
//!     /// Option<SourceAuthor> → Option<DtoAuthor>
//!     #[dto(transform_fn = types::opt_into::<types::SourceAuthor, types::DtoAuthor>)]
//!     author: Option<types::DtoAuthor>,
//!
//!     published_at: Option<DateTime<Utc>>,
//! }
//! ```
//!
//! ### Error Messages
//!
//! The derive macro produces clear, span-accurate diagnostics for common mistakes:
//! - Missing struct attribute: `#[dto(from = Type)]`.
//! - Unsupported item shapes: only named-field structs are supported (tuple/unit structs and enums are rejected).
//! - Unknown field attribute keys: reports the unknown key and the allowed set (`rename`, `transform_fn`, `skip`, `into`).
//! - Duplicate attributes on a field: `rename`, `transform_fn`, `skip`, or `into` repeated.
//! - Conflicting attributes on a field: `skip` cannot appear with any other attribute; `transform_fn` conflicts with `into`.
//! - Invalid `rename` value: empty string is rejected.
//! - Unknown/duplicate struct-level keys: only `from` is allowed at the struct level.
//!
//! See `tests/ui` for compile-fail cases that exercise each diagnostic.
//!
//! ### Limitations
//!
//! - **Named-field structs only**: tuple/unit structs and enums are not supported.
//! - **Structs only**: traits/unions/enums cannot derive `DtoFrom`.
//! - **Owned-only mapping**: generates `impl From<Source> for Target` (no zero-copy/by-ref mode).
//! - **`transform_fn` signature**: must be `FnOnce(SourceFieldType) -> FieldType` (owned input, owned output).
//! - **`into` requires `From`**: `From<SourceFieldType> for FieldType` must exist.
//! - **`skip` requires `Default`**: the target field type must implement `Default`.
//! - **No automatic element mapping**: collections/options do not map inner elements automatically; use `transform_fn`.
//! - **No `auto_into` / `try_into`**: conversions are explicit per-field with `#[dto(into)]`.
//! - **Field existence is validated by the compiler**: a missing/renamed source field causes a compile error at the attribute span.
//!
//! ### Mapping Rules (at a glance)
//!
//! - **Default (owned move)**  
//!   - Same **name** & same **type** → `target = source.field`  
//!   - “Compatible type” means:
//!     - Identical type, or
//!     - `#[dto(into)]` where `From<SourceFieldType> for FieldType` exists, or
//!     - `#[dto(transform_fn = ...)]` provides an explicit conversion
//!
//! - **Field attributes**
//!   - `#[dto(rename = "orig_name")]`  
//!     Reads from a **different source field name** (type must still be compatible).
//!
//!   - `#[dto(transform_fn = path::to::function)]`  
//!     Calls `function(source.orig_name)` before assignment.  
//!     Signature: `FnOnce(SourceFieldType) -> FieldType`.
//!
//!   - `#[dto(skip)]`  
//!     Skips mapping; initializes the field with `Default::default()`.
//!
//!   - `#[dto(into)]`  
//!     Calls `::core::convert::Into::into(source.orig_name)`.  
//!     Requires `From<SourceFieldType> for FieldType` and is infallible.
//!
//! - **Struct attribute (required)**  
//!   - `#[dto(from = Type)]` — Specifies the **source struct** for the mapping.
//!
//! Violations of these rules cause **compile-time errors** with span-accurate diagnostics (see the “Error Messages” section).

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Fields, Ident, Path};

#[derive(Default)]
struct FieldAttrs {
    rename: Option<Ident>,
    rename_span: Option<Span>,
    transform_fn: Option<Path>,
    skip: bool,
    into_flag: bool,
}

enum FieldAction {
    Skip,
    Transform(Path),
    Into,
    Direct,
}

fn decide_action(a: &FieldAttrs) -> FieldAction {
    if a.skip {
        FieldAction::Skip
    } else if let Some(ref f) = a.transform_fn {
        FieldAction::Transform(f.clone())
    } else if a.into_flag {
        FieldAction::Into
    } else {
        FieldAction::Direct
    }
}

#[proc_macro_derive(DtoFrom, attributes(dto))]
pub fn dto_from_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let target_struct = &input.ident;

    let source_ty = match find_source_type(&input.attrs) {
        Ok(path) => path,
        Err(e) => return e.to_compile_error().into(),
    };

    let generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return syn::Error::new_spanned(
                    &input.ident,
                    "DtoFrom only supports named-field structs.",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input.ident, "DtoFrom only supports structs.")
                .to_compile_error()
                .into();
        }
    };

    let field_map = fields.iter().map(|f| {
        let ident = f.ident.as_ref().expect("named fields guaranteed");
        let cfg = match extract_dto_field_attrs(&f.attrs) {
            Ok(c) => c,
            Err(e) => return e.to_compile_error(),
        };
        let src_ident = cfg.rename.clone().unwrap_or_else(|| ident.clone());
        let access_span = cfg.rename_span.unwrap_or_else(|| ident.span());
        generate_field_mapping(ident, &src_ident, &cfg, access_span)
    });

    let owned_impl = quote! {
        impl #impl_generics From<#source_ty> for #target_struct #ty_generics #where_clause {
            fn from(source: #source_ty) -> Self {
                Self { #(#field_map,)* }
            }
        }
    };

    TokenStream::from(quote! { #owned_impl })
}

fn generate_field_mapping(
    ident: &Ident,
    source_ident: &Ident,
    a: &FieldAttrs,
    access_span: Span,
) -> proc_macro2::TokenStream {
    match decide_action(a) {
        FieldAction::Skip => {
            quote! { #ident: Default::default() }
        }
        FieldAction::Transform(ref f) => {
            quote_spanned! { access_span => #ident: #f(source.#source_ident) }
        }
        FieldAction::Into => {
            quote_spanned! { access_span => #ident: ::core::convert::Into::into(source.#source_ident) }
        }
        FieldAction::Direct => {
            quote_spanned! { access_span => #ident: source.#source_ident }
        }
    }
}

fn extract_dto_field_attrs(attrs: &[Attribute]) -> syn::Result<FieldAttrs> {
    let mut cfg = FieldAttrs::default();
    let mut seen_rename = false;
    let mut seen_transform = false;
    let mut seen_skip = false;
    let mut seen_into = false;

    for attr in attrs {
        if !attr.path().is_ident("dto") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let lit = meta.value()?.parse::<syn::LitStr>()?;
                if lit.value().trim().is_empty() {
                    return Err(syn::Error::new(lit.span(), "`rename` cannot be empty"));
                }
                if seen_rename {
                    return Err(syn::Error::new(lit.span(), "duplicate `rename`"));
                }
                seen_rename = true;
                cfg.rename_span = Some(lit.span());
                cfg.rename = Some(Ident::new(&lit.value(), lit.span()));
            } else if meta.path.is_ident("transform_fn") {
                if seen_transform {
                    return Err(syn::Error::new(
                        meta.path.span(),
                        "duplicate `transform_fn`",
                    ));
                }
                seen_transform = true;
                let val = meta.value()?;
                cfg.transform_fn = Some(val.parse()?);
            } else if meta.path.is_ident("skip") {
                if seen_skip {
                    return Err(syn::Error::new(meta.path.span(), "duplicate `skip`"));
                }
                seen_skip = true;
                cfg.skip = true;
            } else if meta.path.is_ident("into") {
                if seen_into {
                    return Err(syn::Error::new(meta.path.span(), "duplicate `into`"));
                }
                seen_into = true;
                cfg.into_flag = true;
            } else {
                return Err(syn::Error::new(
                    meta.path.span(),
                    "unknown #[dto(...)] key; expected one of: rename, transform_fn, skip, into",
                ));
            }
            Ok(())
        })?;
    }

    if cfg.skip && (cfg.rename.is_some() || cfg.transform_fn.is_some() || cfg.into_flag) {
        return Err(syn::Error::new(
            Span::call_site(),
            "`#[dto(skip)]` cannot be combined with `rename`, `transform_fn`, or `into`",
        ));
    }
    if cfg.transform_fn.is_some() && cfg.into_flag {
        return Err(syn::Error::new(
            Span::call_site(),
            "`#[dto(transform_fn = ...)]` conflicts with `#[dto(into)]`",
        ));
    }

    Ok(cfg)
}

fn find_source_type(attrs: &[Attribute]) -> syn::Result<Path> {
    let mut result: Option<Path> = None;
    let mut seen_from = false;
    for attr in attrs {
        if !attr.path().is_ident("dto") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("from") {
                if seen_from {
                    return Err(syn::Error::new(
                        meta.path.span(),
                        "duplicate `from` on struct",
                    ));
                }
                let path: Path = meta.value()?.parse()?;
                result = Some(path);
                seen_from = true;
            } else {
                return Err(syn::Error::new(
                    meta.path.span(),
                    "unknown struct-level #[dto(...)] key; expected `from`",
                ));
            }
            Ok(())
        })?;
    }
    result.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "Expected `#[dto(from = Type)]` on the struct.",
        )
    })
}
