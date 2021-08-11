// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{punctuated::Punctuated, token::Comma, Field};

/// Values of this type contain the information necessary to build either the bodies of the methods
/// for implementing [`Packable`] for a struct or the bodies of the branches for implementing
/// [`Packable`] for a variant of an enum.
///
/// Given that this type can be used for either a struct or a variant we will use the term "record"
/// to refer to both.
pub(crate) struct Fragments {
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
    pub(crate) fn new<const NAMED: bool>(
        name: TokenStream,
        fields: &Punctuated<Field, Comma>,
        pack_error_with: &TokenStream,
        unpack_error_with: &TokenStream,
    ) -> Self {
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
            // <T>::pack(&field_0, packer).map_err(|err| err.map(core::convert::identity).coerce()?;
            // <V>::pack(&field_1, packer).map_err(|err| err.map(core::convert::identity).coerce()?;
            // Ok(())
            // ```
            pack: quote! {
                #(<#types>::pack(#values, packer).map_err(|err| err.map(#pack_error_with)).coerce()?;) *
                Ok(())
            },
            // This would be `0 + <T>::packed_len(&field_0) + <V>::packed_len(&field_1)`. The `0`
            // is used in case the record has no fields.
            packed_len: quote!(0 #(+ <#types>::packed_len(#values))*),
            // And this would be
            // ```
            // Ok(Foo {
            //     bar: <T>::unpack(unpacker).map_err(|err| err.map(core::convert::identity).coerce()?,
            //     baz: <V>::unpack(unpacker).map_err(|err| err.map(core::convert::identity).coerce()?,
            // })```
            unpack: quote! {Ok(#name {
                #(#labels: <#types>::unpack(unpacker).map_err(|err| err.map(#unpack_error_with)).coerce()?,)*
            })},
        }
    }

    /// Consumes the fragments assuming that the record is a struct.
    ///
    /// The returned streams correspond to the bodies of `pack`, `packed_len` and `unpack`.
    pub(crate) fn consume_for_struct(self) -> (TokenStream, TokenStream, TokenStream) {
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
        // <T>::pack(&field_0, packer).coerce()?;
        // <V>::pack(&field_1, packer).coerce()?;
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
    pub(crate) fn consume_for_variant(
        self,
        tag: impl ToTokens,
        tag_ty: impl ToTokens,
    ) -> (TokenStream, TokenStream, TokenStream) {
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
        //     (tag as W).pack(packer).infallible()?;
        //     <T>::pack(&field_0, packer).map_err(|err| err.map(core::convert::identity).coerce()?;
        //     <V>::pack(&field_1, packer).map_err(|err| err.map(core::convert::identity).coerce()?;
        //     Ok(())
        // }
        // ```
        // The cast to `W` is included because `tag` is an integer without type annotations.
        let pack = quote! {
            #pattern => {
                (#tag as #tag_ty).pack(packer).infallible()?;
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
        //     bar: <T>::unpack(unpacker).map_err(|err| err.map(core::convert::identity).coerce()?,
        //     baz: <V>::unpack(unpacker).map_err(|err| err.map(core::convert::identity).coerce()?,
        // })
        // ```
        let unpack = quote!(#tag => #unpack);

        (pack, packed_len, unpack)
    }
}