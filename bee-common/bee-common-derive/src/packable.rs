// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Attribute, Expr, Field, Fields, Generics, Ident, Index, Token, Type, Variant,
};

/// The names of the types that can be used for tags.
const VALID_TAG_TYPES: &[&str] = &["u8", "u16", "u32", "u64"];

/// Values of this type contain the information necessary to build either the bodies of the methods
/// for implementing `Packable` for a struct or the bodies of the branches for implementing
/// `Packable` for a variant of an enum.
///
/// Given that this type can be used for either a struct or a variant we will use the term "record"
/// to refer to both.
struct Fragments {
    // The pattern used to destructure the record.
    pattern: TokenStream,
    // An expression that packs the record.
    pack: TokenStream,
    // An expression that returns the packed length of the record.
    packed_len: TokenStream,
    // An expresion that unpacks the record.
    unpack: TokenStream,
}

impl Fragments {
    /// Create a new set of fragments from the fields of a record with name `name` and fields
    /// `fields`. The `NAMED` parameter specifies if the fields of the record are named or not.
    fn new<const NAMED: bool>(name: TokenStream, fields: &Punctuated<Field, Comma>) -> Self {
        let len = fields.len();
        // The type of each field of the record.
        let mut types = Vec::with_capacity(len);
        // The label of each field of the record.
        let mut labels = Vec::with_capacity(len);
        // The value of each field of the record.
        let mut values = Vec::with_capacity(len);

        for (index, Field { ident, ty, .. }) in fields.iter().enumerate() {
            if NAMED {
                // This is a named field, which means its `ident` cannot be `None`.
                labels.push(ident.as_ref().unwrap().to_token_stream());
            } else {
                // This is an unnamed field. We push the index because in Rust `Foo(T)` is
                // equivalent to `Foo { 0: T }`, which allows us to handle both cases
                // homogeneously.
                labels.push(proc_macro2::Literal::u64_unsuffixed(index as u64).to_token_stream());
            }
            types.push(ty);
            // We will use variables called `field_<index>` for the values of each field.
            values.push(format_ident!("field_{}", index));
        }

        // Assume that the current record is `Foo { bar: T, baz: V }`.
        Self {
            // This would be `Foo { bar: field_0 , baz: field_1 }`.
            pattern: quote!(#name { #(#labels: #values),* }),
            // This would be
            // ```
            // <T>::pack(&field_0, packer)?;
            // <V>::pack(&field_1, packer)?;
            // Ok(())
            // ```
            pack: quote! {
                #(<#types>::pack(&#values, packer)?;) *
                Ok(())
            },
            // This would be `0 + <T>::packed_len(&field_0) + <V>::packed_len(&field_1)`. The `0`
            // is used in case the record has no fields.
            packed_len: quote!(0 #(+ <#types>::packed_len(#values))*),
            // And this would be
            // ```
            // Ok(Foo {
            //     bar: <T>::unpack(unpacker).map_err(|x| x.coerce())?,
            //     baz: <V>::unpack(unpacker).map_err(|x| x.coerce())?,
            // })```
            unpack: quote! {Ok(#name {
                #(#labels: <#types>::unpack(unpacker).map_err(|x| x.coerce())?,)*
            })},
        }
    }

    /// Consumes the fragments assuming that the record is a struct.
    ///
    /// The returned streams correspond to the bodies of `pack`, `packed_len` and `unpack`.
    fn consume_for_struct(self) -> (TokenStream, TokenStream, TokenStream) {
        let Self {
            pattern,
            pack,
            packed_len,
            unpack,
        } = self;
        // Following the example from `new`. This would be
        // ```
        // let Foo {
        //     bar: field_0,
        //     baz: field_1,
        // } = self;
        // <T>::pack(&field_0, packer)?;
        // <V>::pack(&field_1, packer)?;
        // Ok(())
        // ```
        // The whole destructuring thing is done so we can do both variants and structs with the
        // same fragments even though it would be more natural to use `self.bar` and `self.baz`
        // instead.
        let pack = quote! {
            let #pattern = self;
            #pack
        };
        // And this would be
        // ```
        // let Foo {
        //     bar: field_0,
        //     baz: field_1,
        // } = self;
        // 0 + <T>::packed_len(&field_0) + <V>::packed_len(&field_1)
        // ```
        let packed_len = quote! {
            let #pattern = self;
            #packed_len
        };

        (pack, packed_len, unpack)
    }

    /// Consumes the fragments assuming that the record is a variant.
    ///
    /// The returned streams correspond to the branches necessary to implement `pack`, `packed_len`
    /// and `unpack` for this variant.
    fn consume_for_variant(self, tag: impl ToTokens, tag_ty: impl ToTokens) -> (TokenStream, TokenStream, TokenStream) {
        let Self {
            pattern,
            pack,
            packed_len,
            unpack,
        } = self;
        // Following the example from `new` and assuming that the tag for this variant is `tag` and
        // the type of the tag is `W`. This would be
        // ```
        // Foo { bar: field_0 , baz: field_1 } => {
        //     (tag as W).pack(packer)?;
        //     <T>::pack(&field_0, packer)?;
        //     <V>::pack(&field_1, packer)?;
        //     Ok(())
        // }
        // ```
        // The cast to `W` is included because `tag` is an integer without type annotations.
        let pack = quote! {
            #pattern => {
                (#tag as #tag_ty).pack(packer)?;
                #pack
            }
        };

        // This would be
        // ```
        // Foo { bar: field_0 , baz: field_1 } => (tag as W).packed_len() + 0 + <T>::packed_len(&field_0) + <V>::packed_len(&field_1)
        // ```
        let packed_len = quote!(#pattern => (#tag as #tag_ty).packed_len() + #packed_len);

        // And this would be
        // ```
        // tag => Ok(Foo {
        //     bar: <T>::unpack(unpacker).map_err(|x| x.coerce())?,
        //     baz: <V>::unpack(unpacker).map_err(|x| x.coerce())?,
        // })
        // ```
        let unpack = quote!(#tag => #unpack);

        (pack, packed_len, unpack)
    }
}

/// Generate the bodies of `pack`, `packed_len` and `unpack` for a struct with fields `fields`.
pub(crate) fn gen_bodies_for_struct(fields: Fields) -> (TokenStream, TokenStream, TokenStream) {
    match fields {
        Fields::Named(fields) => Fragments::new::<true>(quote!(Self), &fields.named).consume_for_struct(),
        Fields::Unnamed(fields) => Fragments::new::<false>(quote!(Self), &fields.unnamed).consume_for_struct(),
        Fields::Unit => (quote!(Ok(())), quote!(0), quote!(Ok(Self))),
    }
}

/// Generate the bodies of `pack`, `packed_len` and `unpack` for a enum with variants `variants`
/// and tag type `tag_ty`.
pub(crate) fn gen_bodies_for_enum(
    variants: &Punctuated<Variant, Comma>,
    tag_ty: Type,
) -> (TokenStream, TokenStream, TokenStream) {
    let len = variants.len();

    // Validate that the tag type is in `VALID_TAG_TYPES`.
    match &tag_ty {
        Type::Path(ty_path) if VALID_TAG_TYPES.iter().any(|ty| ty_path.path.is_ident(ty)) => (),
        _ => abort!(tag_ty.span(), "Tags for enums can only be unsigned, sized integers."),
    }

    // Store the tags and names of the variants so we can guarantee that tags are unique.
    let mut tags = Vec::<(Index, &Ident)>::with_capacity(len);

    // The branch for packing each variant.
    let mut pack_branches = Vec::with_capacity(len);
    // The branch with the packing length of each variant.
    let mut packed_len_branches = Vec::with_capacity(len);
    // The branch for unpacking each variant.
    let mut unpack_branches = Vec::with_capacity(len);

    for variant in variants {
        let Variant {
            attrs, ident, fields, ..
        } = variant;

        // Verify that this variant has a `"tag"` attribute with an untyped, unsigned integer on it.
        let curr_tag = if let Some(tag) = parse_attr::<Index>("tag", attrs) {
            tag
        } else {
            // Try to parse a `"tag"` attribute with any expression for good error reporting.
            let expr = parse_attr::<Expr>("tag", attrs).unwrap_or_else(|| {
                // There is no tag attribute at all.
                abort!(
                    ident.span(),
                    "All variants of an enum that derives `Packable` require a `#[packable(tag = ...)]` attribute."
                )
            });
            // There is a tag attribute with an illegal expression.
            abort!(
                expr.span(),
                "Tags for variants can only be integers without type annotations.",
            );
        };

        // Search for the current tag inside `tags`.
        match tags.binary_search_by(|(tag, _)| tag.index.cmp(&curr_tag.index)) {
            // If we already have this tag, then it is duplicated, we error reporting the name of
            // the variant using it.
            Ok(pos) => abort!(
                curr_tag.span,
                "The tag `{}` is already being used for the `{}` variant.",
                curr_tag.index,
                tags[pos].1
            ),
            // If we do not have this tag, we store it.
            Err(pos) => tags.insert(pos, (curr_tag.clone(), ident)),
        }

        // Keep only the index for the current tag.
        let tag = proc_macro2::Literal::u64_unsuffixed(curr_tag.index as u64);

        // Add the `Self::` prefix to the name of the variant.
        let name = quote!(Self::#ident);

        let (pack_branch, packed_len_branch, unpack_branch) = match fields {
            Fields::Named(fields) => Fragments::new::<true>(name, &fields.named).consume_for_variant(tag, &tag_ty),
            Fields::Unnamed(fields) => Fragments::new::<false>(name, &fields.unnamed).consume_for_variant(tag, &tag_ty),
            Fields::Unit => (
                quote!(#name => (#tag as #tag_ty).pack(packer)),
                quote!(#name => (#tag as #tag_ty).packed_len()),
                quote!(#tag => Ok(#name)),
            ),
        };

        pack_branches.push(pack_branch);
        packed_len_branches.push(packed_len_branch);
        unpack_branches.push(unpack_branch);
    }

    // Add a surrounding match expresison for the branches.
    (
        quote! {
            match self {
                #(#pack_branches,) *
            }
        },
        quote! {
            match self {
                #(#packed_len_branches,) *
            }
        },
        quote! {
            match unpacker.unpack_infallible::<#tag_ty>()? {
                #(#unpack_branches,) *
                tag => Err(bee_common::packable::UnpackError::Packable(bee_common::packable::UnknownTagError(tag).into()))
            }
        },
    )
}

/// Generate the implementation of `Packable`.
pub(crate) fn gen_impl(
    ident: &Ident,
    generics: &Generics,
    error_type: TokenStream,
    pack_body: TokenStream,
    packed_len_body: TokenStream,
    unpack_body: TokenStream,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics Packable for #ident #ty_generics #where_clause {
            type Error = #error_type;

            fn pack<P: bee_common::packable::Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
                #pack_body
            }

            fn unpack<U: bee_common::packable::Unpacker>(unpacker: &mut U) -> Result<Self, bee_common::packable::UnpackError<Self::Error, U::Error>> {
                #unpack_body
            }

            fn packed_len(&self) -> usize {
                #packed_len_body
            }
        }
    }
}

/// Utility function to parse an attribute of the form `#[packable(ident = value)]` where `value`
/// is of type `T` from an slice of attributes. Return `Some(value)` if such attribute exists and
/// return `None` otherwise.
pub(crate) fn parse_attr<T: Parse>(ident: &str, attrs: &[Attribute]) -> Option<T> {
    struct AttrArg<T> {
        ident: Ident,
        value: T,
    }

    impl<T: Parse> Parse for AttrArg<T> {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let ident = input.parse::<Ident>()?;
            let _ = input.parse::<Token![=]>()?;
            let value = input.parse::<T>()?;

            Ok(Self { ident, value })
        }
    }

    for attr in attrs {
        if attr.path.is_ident("packable") {
            match attr.parse_args::<AttrArg<T>>() {
                Ok(arg) if arg.ident == ident => return Some(arg.value),
                _ => (),
            }
        }
    }

    None
}
