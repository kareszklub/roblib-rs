extern crate proc_macro;

use proc_macro2::TokenStream;
use proc_macro_error::{abort, proc_macro_error};

use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput};

fn get_readable(struct_ident: &proc_macro2::Ident, struct_data: &syn::DataStruct) -> TokenStream {
    let mut binary = TokenStream::new();
    let mut binary_async = TokenStream::new();
    let mut text = TokenStream::new();

    let mut field_names = TokenStream::new();
    let mut tuple = false;

    for (i, field) in struct_data.fields.iter().enumerate() {
        let name = if let Some(name) = &field.ident {
            tuple = false;
            name.clone()
        } else {
            tuple = true;
            format_ident!("_{i}")
        };

        field_names.append_all(quote! { #name, });
        text.append_all(quote! {
            let #name = Readable::parse_text(s)?;
        });
        binary.append_all(quote! {
            let #name = Readable::parse_binary(r)?;
        });
        binary_async.append_all(quote! {
            let #name = Readable::parse_binary_async(r).await?;
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
        #[cfg_attr(feature = "async", async_trait::async_trait)]
        impl Readable for #struct_ident {
            fn parse_text<'a>(s: &mut impl ::std::iter::Iterator<Item = &'a str>) -> ::anyhow::Result<Self> {
                #text
                Ok(#ret)
            }
            fn parse_binary(r: &mut impl ::std::io::Read) -> ::anyhow::Result<Self> {
                #binary
                Ok(#ret)
            }
            #[cfg(feature = "async")]
            async fn parse_binary_async(r: &mut (impl ::futures::AsyncRead + Send + Unpin)) -> ::anyhow::Result<Self> {
                #binary_async
                Ok(#ret)
            }
        }
    };
    // println!("{res}");
    res
}

#[proc_macro_error]
#[proc_macro_derive(Readable)]
pub fn derive_readable(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inp = parse_macro_input!(item as DeriveInput);

    let DeriveInput {
        ident,
        data: Data::Struct(struct_data),
        ..
    } = inp else {
        abort!(inp.span(), "You can only use this on a struct")
    };

    get_readable(&ident, &struct_data).into()
}

#[proc_macro_error]
#[proc_macro_derive(Writable)]
pub fn derive_writable(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inp = parse_macro_input!(item as DeriveInput);

    let DeriveInput {
        ident: struct_ident,
        data: Data::Struct(struct_data),
        ..
    } = inp else {
        abort!(inp.span(), "You can only use this on a struct")
    };

    get_writable(&struct_ident, &struct_data).into()
}

fn get_writable(struct_ident: &proc_macro2::Ident, struct_data: &syn::DataStruct) -> TokenStream {
    let mut binary_async = TokenStream::new();
    let mut binary = TokenStream::new();
    let mut text = TokenStream::new();

    for (i, field) in struct_data.fields.iter().enumerate() {
        let name = if let Some(name) = &field.ident {
            name.to_token_stream()
        } else {
            let i = syn::Index::from(i);
            quote! { self.#i }
        };

        text.append_all(quote! {
            #name.write_text(f)?;
        });
        binary.append_all(quote! {
            #name.write_binary(w)?;
        });
        binary_async.append_all(quote! {
            #name.write_binary_async(w).await?;
        });
    }

    let res = quote! {
        #[cfg_attr(feature = "async", async_trait::async_trait)]
        impl Writable for #struct_ident {
            fn write_text(&self, f: &mut dyn ::std::ops::FnMut(&str)) -> ::std::fmt::Result {
                #text
                Ok(())
            }
            fn write_binary(&self, w: &mut dyn ::std::io::Write) -> ::anyhow::Result<()> {
                #binary
                Ok(())
            }
            #[cfg(feature = "async")]
            async fn write_binary_async(&self, w: &mut (dyn ::futures::AsyncWrite + Send + Unpin)) -> ::anyhow::Result<()> {
                #binary_async
                Ok(())
            }
        }
    };
    // println!("{res}");
    res
}

#[proc_macro_error]
#[proc_macro_derive(Command)]
pub fn derive_command(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inp = parse_macro_input!(item as DeriveInput);

    let DeriveInput {
        ident: struct_ident,
        data: Data::Struct(struct_data),
        ..
    } = inp else {
        abort!(inp.span(), "You can only use this on a struct")
    };

    let readable = get_readable(&struct_ident, &struct_data);
    let writable = get_writable(&struct_ident, &struct_data);

    let res = quote! {
        #readable
        #writable

        impl From<#struct_ident> for crate::cmd::Concrete {
            fn from(value: #struct_ident) -> Self {
                crate::cmd::Concrete::#struct_ident(value)
            }
        }
        impl From<crate::cmd::Concrete> for #struct_ident {
            fn from(value: crate::cmd::Concrete) -> Self {
                if let crate::cmd::Concrete::#struct_ident(m) = value {
                    m
                } else {
                    panic!("Tried to convert an unknown command to a concrete command")
                }
            }
        }
    };

    // println!("{res}");
    proc_macro::TokenStream::from(res)
}
