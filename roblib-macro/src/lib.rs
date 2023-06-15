// FIXME: REMOVE
#![allow(unused)]

extern crate proc_macro;

use proc_macro2::TokenStream;
use proc_macro_error::{abort, emit_error, proc_macro_error};

use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{
    parse2, parse_macro_input, parse_str, spanned::Spanned, Attribute, Data, DeriveInput, Lit,
    LitStr, Type,
};

fn nth_name(m: usize) -> char {
    (b'a' + m as u8) as char
}

// TODO: implement derive macro for Readable
#[proc_macro_error]
#[proc_macro_derive(Readable)]
pub fn derive_readable(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inp = parse_macro_input!(item as DeriveInput);

    let DeriveInput {
        ident: enum_ident,
        data: Data::Struct(struct_data),
        ..
    } = inp else {
        abort!(inp.span(), "You can only use this on an enum")
    };

    let mut from_text_impl = TokenStream::new();
    let mut read_impl = TokenStream::new();

    let res = quote! {
        impl Readable for #enum_ident {
            fn parse_str<'a>(s: &mut impl ::std::iter::Iterator<Item = &'a str>) -> ::anyhow::Result<Self> {
                #from_text_impl
                todo!()
            }
            fn parse_binary(r: &mut impl ::std::io::Read) -> ::anyhow::Result<Self> {
                #read_impl
                todo!()
            }
        }
    };

    proc_macro::TokenStream::from(res)
}

// TODO: implement derive macro for Writable
#[proc_macro_error]
#[proc_macro_derive(Writable)]
pub fn derive_writable(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inp = parse_macro_input!(item as DeriveInput);

    let DeriveInput {
        ident: enum_ident,
        data: Data::Struct(struct_data),
        ..
    } = inp else {
        abort!(inp.span(), "You can only use this on an enum")
    };

    let mut display_impl = TokenStream::new();
    let mut write_impl = TokenStream::new();

    let res = quote! {
        impl Writable for #enum_ident {
            fn write_str(&self, f: &mut dyn ::std::fmt::Write) -> ::std::fmt::Result {
                #display_impl
                todo!()
            }
            fn write_binary(&self, w: &mut dyn ::std::io::Write) -> ::anyhow::Result<()> {
                #write_impl
                todo!()
            }
        }
    };

    proc_macro::TokenStream::from(res)
}
