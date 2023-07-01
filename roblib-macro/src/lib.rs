extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::{abort, proc_macro_error};

use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{parse_macro_input, Data, DeriveInput};

fn get_readable_enum(ident: &Ident, data: &syn::DataEnum) -> TokenStream {
    let mut binary_async = TokenStream::new();
    let mut binary = TokenStream::new();
    let mut text = TokenStream::new();

    let tag_type = Ident::new(
        match data.variants.len() {
            0 => abort!(
                ident.span(),
                "Can't implement ::roblib_parsing::Readable for enum without variants"
            ),
            0x01..=0xFF => "u8",
            0x01_00..=0xFF_FF => "u16",
            0x01_00_00..=0xFF_FF_FF_FF => "u32",
            0x01_00_00_00_00..=0xFF_FF_FF_FF_FF_FF_FF_FF => "u64",
            _ => "u128",
        },
        proc_macro2::Span::mixed_site(),
    );

    for (i, variant) in data.variants.iter().enumerate() {
        let variant_name = &variant.ident;

        let mut variant_args = TokenStream::new();

        let mut variant_binary_async = TokenStream::new();
        let mut variant_binary = TokenStream::new();
        let mut variant_text = TokenStream::new();

        enum ArgsType {
            Empty,
            Tuple,
            Fields,
        }

        let mut args_type = ArgsType::Empty;
        for (i, field) in variant.fields.iter().enumerate() {
            let name = if let Some(name) = &field.ident {
                args_type = ArgsType::Fields;
                name.to_token_stream()
            } else {
                args_type = ArgsType::Tuple;
                let name = format_ident!("_{i}");
                quote! { #name }
            };
            variant_args.append_all(quote! { #name, });

            let ty = &field.ty;

            variant_text.append_all(quote! {
                let #name: #ty = ::roblib_parsing::Readable::parse_text(s)?;
            });
            variant_binary.append_all(quote! {
                let #name: #ty = ::roblib_parsing::Readable::parse_binary(r)?;
            });
            variant_binary_async.append_all(quote! {
                let #name: #ty = ::roblib_parsing::Readable::parse_binary_async(r).await?;
            });
        }

        match args_type {
            ArgsType::Fields => variant_args = quote! { { #variant_args } },
            ArgsType::Tuple => variant_args = quote! { ( #variant_args ) },

            _ => (),
        }

        let tag = syn::LitInt::new(&format!("{i}{tag_type}"), Span::mixed_site());

        text.append_all(quote! {
            #tag => {
                #variant_text
                ::std::result::Result::Ok(Self :: #variant_name #variant_args)
            },
        });
        binary.append_all(quote! {
            #tag => {
                #variant_binary
                ::std::result::Result::Ok(Self :: #variant_name #variant_args)
            },
        });
        binary_async.append_all(quote! {
            #tag => {
                #variant_binary_async
                ::std::result::Result::Ok(Self :: #variant_name #variant_args)
            },
        });
    }

    let res = quote! {
        #[cfg_attr(feature = "async", ::async_trait::async_trait)]
        impl ::roblib_parsing::Readable for #ident {
            fn parse_text<'a>(s: &mut impl ::std::iter::Iterator<Item = &'a str>) -> ::anyhow::Result<Self> {
                let tag: #tag_type = ::roblib_parsing::Readable::parse_text(s)?;
                match tag {
                    #text
                    _ => ::std::result::Result::Err(::anyhow::Error::msg("Unknown enum variant"))
                }
            }
            fn parse_binary(r: &mut impl ::std::io::Read) -> ::anyhow::Result<Self> {
                let tag: #tag_type = ::roblib_parsing::Readable::parse_binary(r)?;
                match tag {
                    #binary
                    _ => ::std::result::Result::Err(::anyhow::Error::msg("Unknown enum variant"))
                }
            }
            #[cfg(feature = "async")]
            async fn parse_binary_async(r: &mut (impl ::futures::AsyncRead + Send + Unpin)) -> ::anyhow::Result<Self> {
                let tag: #tag_type = ::roblib_parsing::Readable::parse_binary_async(r).await?;
                match tag {
                    #binary_async
                    _ => ::std::result::Result::Err(::anyhow::Error::msg("Unknown enum variant"))
                }
            }
        }
    };
    // println!("{res}");
    res
}

fn get_readable_struct(struct_ident: &Ident, struct_data: &syn::DataStruct) -> TokenStream {
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
            let #name = ::roblib_parsing::Readable::parse_text(s)?;
        });
        binary.append_all(quote! {
            let #name = ::roblib_parsing::Readable::parse_binary(r)?;
        });
        binary_async.append_all(quote! {
            let #name = ::roblib_parsing::Readable::parse_binary_async(r).await?;
        });
    }

    let ret = if tuple {
        quote! { Self(#field_names) }
    } else {
        quote! { Self { #field_names } }
    };

    let res = quote! {
        #[cfg_attr(feature = "async", ::async_trait::async_trait)]
        impl ::roblib_parsing::Readable for #struct_ident {
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
    get_readable(&inp.ident, &inp.data).into()
}

fn get_readable(ident: &Ident, data: &Data) -> TokenStream {
    match data {
        Data::Struct(d) => get_readable_struct(ident, d),
        Data::Enum(e) => get_readable_enum(ident, e),
        Data::Union(u) => {
            abort!(u.union_token.span, "Unions aren't supported");
        }
    }
}

fn get_writable(ident: &Ident, data: &Data) -> TokenStream {
    match data {
        Data::Struct(d) => get_writable_struct(ident, d),
        Data::Enum(e) => get_writable_enum(ident, e),
        Data::Union(u) => {
            abort!(u.union_token.span, "Unions aren't supported");
        }
    }
}

#[proc_macro_error]
#[proc_macro_derive(Writable)]
pub fn derive_writable(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inp = parse_macro_input!(item as DeriveInput);
    get_writable(&inp.ident, &inp.data).into()
}

fn get_writable_struct(struct_ident: &Ident, struct_data: &syn::DataStruct) -> TokenStream {
    let mut binary_async = TokenStream::new();
    let mut binary = TokenStream::new();
    let mut text = TokenStream::new();

    for (i, field) in struct_data.fields.iter().enumerate() {
        let name = if let Some(name) = &field.ident {
            name.to_token_stream()
        } else {
            let i = syn::Index::from(i);
            quote! { #i }
        };

        text.append_all(quote! {
            ::roblib_parsing::Writable::write_text(&self. #name, w)?;
        });
        binary.append_all(quote! {
            ::roblib_parsing::Writable::write_binary(&self. #name, w)?;
        });
        binary_async.append_all(quote! {
            ::roblib_parsing::Writable::write_binary_async(&self. #name, w).await?;
        });
    }

    let res = quote! {
        #[cfg_attr(feature = "async", ::async_trait::async_trait)]
        impl ::roblib_parsing::Writable for #struct_ident {
            fn write_text(&self, w: &mut dyn std::fmt::Write) -> ::std::fmt::Result {
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

fn get_writable_enum(ident: &Ident, data: &syn::DataEnum) -> TokenStream {
    let mut binary_async = TokenStream::new();
    let mut binary = TokenStream::new();
    let mut text = TokenStream::new();

    let tag_type = Ident::new(
        match data.variants.len() {
            0 => abort!(
                ident.span(),
                "Can't implement ::roblib_parsing::Writable for enum without variants"
            ),
            0x01..=0xFF => "u8",
            0x01_00..=0xFF_FF => "u16",
            0x01_00_00..=0xFF_FF_FF_FF => "u32",
            0x01_00_00_00_00..=0xFF_FF_FF_FF_FF_FF_FF_FF => "u64",
            _ => "u128",
        },
        proc_macro2::Span::mixed_site(),
    );

    for (i, variant) in data.variants.iter().enumerate() {
        let variant_name = &variant.ident;

        let mut variant_args = TokenStream::new();

        let mut variant_binary_async = TokenStream::new();
        let mut variant_binary = TokenStream::new();
        let mut variant_text = TokenStream::new();

        enum ArgsType {
            Empty,
            Tuple,
            Fields,
        }

        let mut args_type = ArgsType::Empty;
        for (i, field) in variant.fields.iter().enumerate() {
            let name = if let Some(name) = &field.ident {
                args_type = ArgsType::Fields;
                name.to_token_stream()
            } else {
                args_type = ArgsType::Tuple;
                let name = format_ident!("_{i}");
                quote! { #name }
            };
            variant_args.append_all(quote! { #name, });

            variant_text.append_all(quote! {
                ::roblib_parsing::Writable::write_text(&#name, w)?;
            });
            variant_binary.append_all(quote! {
                ::roblib_parsing::Writable::write_binary(&#name, w)?;
            });
            variant_binary_async.append_all(quote! {
                ::roblib_parsing::Writable::write_binary_async(&#name, w).await?;
            });
        }

        match args_type {
            ArgsType::Fields => variant_args = quote! { { #variant_args } },
            ArgsType::Tuple => variant_args = quote! { ( #variant_args ) },

            _ => (),
        }

        let i = syn::LitInt::new(&format!("{i}{tag_type}"), Span::mixed_site());
        text.append_all(quote! {
            #ident :: #variant_name #variant_args => {
                let tag: #tag_type = #i;
                ::roblib_parsing::Writable::write_text(&tag, w)?;
                #variant_text
            },
        });
        binary.append_all(quote! {
            #ident :: #variant_name #variant_args => {
                let tag: #tag_type = #i;
                ::roblib_parsing::Writable::write_binary(&tag, w)?;
                #variant_binary
            },
        });
        binary_async.append_all(quote! {
            #ident :: #variant_name #variant_args => {
                let tag: #tag_type = #i;
                ::roblib_parsing::Writable::write_binary_async(&tag, w).await?;
                #variant_binary_async
            },
        });
    }

    let res = quote! {
        #[cfg_attr(feature = "async", ::async_trait::async_trait)]
        impl ::roblib_parsing::Writable for #ident {
            fn write_text(&self, w: &mut dyn std::fmt::Write) -> ::std::fmt::Result {
                match self {
                    #text
                }
                Ok(())
            }
            fn write_binary(&self, w: &mut dyn ::std::io::Write) -> ::anyhow::Result<()> {
                match self {
                    #binary
                }
                Ok(())
            }
            #[cfg(feature = "async")]
            async fn write_binary_async(&self, w: &mut (dyn ::futures::AsyncWrite + Send + Unpin)) -> ::anyhow::Result<()> {
                match self {
                    #binary_async
                }
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
    let ident = &inp.ident;

    let readable = get_readable(ident, &inp.data);
    let writable = get_writable(ident, &inp.data);

    let res = quote! {
        #readable
        #writable

        impl From<#ident> for crate::cmd::Concrete {
            fn from(value: #ident) -> Self {
                crate::cmd::Concrete::#ident(value)
            }
        }
        impl From<crate::cmd::Concrete> for #ident {
            fn from(value: crate::cmd::Concrete) -> Self {
                if let crate::cmd::Concrete::#ident(m) = value {
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

#[proc_macro_error]
#[proc_macro_derive(Event)]
pub fn derive_event(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inp = parse_macro_input!(item as DeriveInput);
    let ident = &inp.ident;

    let readable = get_readable(ident, &inp.data);
    let writable = get_writable(ident, &inp.data);

    let res = quote! {
        #readable
        #writable

        impl From<#ident> for crate::event::Concrete {
            fn from(value: #ident) -> Self {
                crate::event::Concrete::#ident(value)
            }
        }
        impl From<crate::event::Concrete> for #ident {
            fn from(value: crate::event::Concrete) -> Self {
                if let crate::event::Concrete::#ident(m) = value {
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
