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

    let mut parse_str_impl = TokenStream::new();
    let mut parse_binary_impl = TokenStream::new();

    let mut field_names = TokenStream::new();
    let mut tuple = false;

    for (i, field) in struct_data.fields.into_iter().enumerate() {
        let name = if let Some(name) = &field.ident {
            tuple = false;
            name.clone()
        } else {
            tuple = true;
            format_ident!("_{i}")
        };

        field_names.append_all(quote! { #name, });
        parse_str_impl.append_all(quote! {
            let #name = Readable::parse_str(s)?;
        });
        parse_binary_impl.append_all(quote! {
            let #name = Readable::parse_binary(r)?;
        });
    }

    let ret = if tuple {
        quote! { Self(#field_names) }
    } else {
        quote! {Self {
            #field_names
        }}
    };

    let res = quote! {
        impl Readable for #enum_ident {
            fn parse_str<'a>(s: &mut impl ::std::iter::Iterator<Item = &'a str>) -> ::anyhow::Result<Self> {
                #parse_str_impl
                Ok(#ret)
            }
            fn parse_binary(r: &mut impl ::std::io::Read) -> ::anyhow::Result<Self> {
                #parse_binary_impl
                Ok(#ret)
            }
        }
    };
    // println!("{res}");
    proc_macro::TokenStream::from(res)
}

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

    let mut write_str_impl = TokenStream::new();
    let mut write_binary_impl = TokenStream::new();

    for (i, field) in struct_data.fields.into_iter().enumerate() {
        let name = if let Some(name) = field.ident {
            name.to_token_stream()
        } else {
            let i = syn::Index::from(i);
            quote! { self.#i }
        };

        write_str_impl.append_all(quote! {
            #name.write_str(f)?;
        });
        write_binary_impl.append_all(quote! {
            #name.write_binary(w)?;
        });
    }

    let res = quote! {
        impl Writable for #enum_ident {
            fn write_str(&self, f: &mut dyn ::std::ops::FnMut(&str)) -> ::std::fmt::Result {
                #write_str_impl
                Ok(())
            }
            fn write_binary(&self, w: &mut dyn ::std::io::Write) -> ::anyhow::Result<()> {
                #write_binary_impl
                Ok(())
            }
        }
    };
    // println!("{res}");
    proc_macro::TokenStream::from(res)
}
