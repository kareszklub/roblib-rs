extern crate proc_macro;
use std::collections::HashSet;

use proc_macro2::TokenStream;
use proc_macro_error::{abort, emit_error, proc_macro_error};

use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{
    parse2, parse_macro_input, parse_str, spanned::Spanned, Attribute, Data, DeriveInput, Lit,
    LitStr, Type,
};

mod parsers;

struct Attributes {
    prefix: Option<char>,
    query: Option<Type>,
}

fn process_atributes(attrs: &[Attribute]) -> Attributes {
    let mut prefix = None;
    let mut query = None;

    for a in attrs {
        let Ok(nv) = a.meta.require_name_value() else {
            continue;
        };
        if let Some(attr_name) = nv.path.get_ident() {
            if attr_name == "prefix" {
                prefix = parse2::<Lit>(nv.value.to_token_stream())
                    .ok()
                    .and_then(|c| match c {
                        Lit::Str(s) => {
                            if s.value().len() == 1 {
                                s.value().chars().next()
                            } else {
                                None
                            }
                        }
                        Lit::ByteStr(s) => {
                            if s.value().len() == 1 {
                                Some(s.value()[0] as char)
                            } else {
                                None
                            }
                        }
                        Lit::Byte(b) => Some(b.value() as char),
                        Lit::Char(c) => Some(c.value()),
                        _ => None,
                    });

                if prefix.is_none() {
                    emit_error!(nv.span(), "Prefix can only be a single character");
                }
            } else if attr_name == "query" {
                let s = parse2::<LitStr>(nv.value.to_token_stream()).ok();
                query = s.and_then(|q| parse_str::<Type>(&q.value()).ok());

                if query.is_none() {
                    emit_error!(nv.span(), "Prefix has to be a valid rust type in a string");
                }
            }
        }
    }

    Attributes { prefix, query }
}

fn nth_name(m: usize) -> char {
    (b'a' + m as u8) as char
}

#[proc_macro_error]
#[proc_macro_derive(Parsable, attributes(prefix, query))]
pub fn derive_parsable(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inp = parse_macro_input!(item as DeriveInput);

    let DeriveInput {
        ident: enum_ident,
        data: Data::Enum(enum_data),
        ..
    } = inp else {
        abort!(inp.span(), "You can only use this on an enum")
    };

    let mut existing_prefixes = HashSet::new();

    let mut display_impl = TokenStream::new();
    let mut write_impl = TokenStream::new();
    let mut from_text_impl = TokenStream::new();
    let mut read_impl = TokenStream::new();

    let mut query_parsers = TokenStream::new();
    let mut has_return = TokenStream::new();

    for variant in enum_data.variants {
        let processed_attibutes = process_atributes(&variant.attrs);

        let prefix: char = if let Some(assigned_prefix) = processed_attibutes.prefix {
            if existing_prefixes.contains(&assigned_prefix) {
                emit_error!(variant.span(), "Prefix already used");
            }
            assigned_prefix
        } else {
            let name = variant.ident.to_string();
            let mut auto_prefix = name
                .to_ascii_lowercase()
                .chars()
                .next()
                .expect("Zero length enum variant name???");

            if existing_prefixes.contains(&auto_prefix) {
                auto_prefix = auto_prefix.to_ascii_lowercase();
                if existing_prefixes.contains(&auto_prefix) {
                    emit_error!(variant.span(), "Ran out of prefixes");
                }
            }

            auto_prefix
        };

        existing_prefixes.insert(prefix);

        if !prefix.is_ascii_alphabetic() {
            emit_error!(variant.span(), "Prefix can only be an ASCII letter");
        }

        let prefix_str = String::from_utf8(vec![prefix as u8]).unwrap();

        let mut fields = vec![];
        let mut args = TokenStream::new();
        for (i, f) in variant.fields.iter().enumerate() {
            let name = format_ident!("{}", nth_name(i));
            args.append(name.clone());
            if i != variant.fields.len() - 1 {
                args.append_all(quote! {,});
            }

            fields.push((name, &f.ty));
        }

        let args_pattern = if fields.is_empty() {
            args.clone()
        } else {
            quote! { (#args) }
        };

        let name = &variant.ident;

        let display = {
            let mut ts = TokenStream::new();
            for (name, ty) in &fields {
                let disp = match parsers::display(ty) {
                    Ok(w) => w,
                    Err(e) => {
                        emit_error!(
                            variant.span(),
                            format!("Couldn't binary writer for argument because: {e}")
                        );
                        continue;
                    }
                };

                ts.append_all(quote! {
                    write!(f, "{}", ARGUMENT_SEPARATOR)?;
                    let i = #name;
                    #disp
                });
            }

            quote! {
                write!(f, #prefix_str)?;
                #ts
            }
        };
        display_impl.append_all(quote! { Cmd::#name #args_pattern => { #display }, });

        let write = {
            let mut ts = quote! { w.write_all(&[#prefix as u8])?; };
            for (name, ty) in &fields {
                let write = match parsers::write(ty) {
                    Ok(w) => w,
                    Err(e) => {
                        emit_error!(
                            variant.span(),
                            format!("Couldn't create binary writer for argument because: {e}")
                        );
                        continue;
                    }
                };

                ts.append_all(quote! {
                    let i = #name;
                    #write
                });
            }
            ts
        };
        write_impl.append_all(quote! { Cmd::#name #args_pattern => { #write }, });

        let from_text = {
            let mut ts = TokenStream::new();

            for (name, ty) in &fields {
                let parsing = match parsers::from_text(ty) {
                    Ok(t) => t,
                    Err(e) => {
                        emit_error!(
                            variant.span(),
                            format!("Couldn't create text parser for argument because: {e}")
                        );
                        continue;
                    }
                };
                ts.append_all(quote! {
                    let #name = #parsing;
                });
            }

            quote! {
                #ts
                ::std::result::Result::Ok(Cmd::#name #args_pattern)
            }
        };
        from_text_impl.append_all(quote! { #prefix_str => { #from_text }, });

        let read = {
            let mut ts = TokenStream::new();

            for (name, ty) in &fields {
                let reading = match parsers::read(ty) {
                    Ok(read) => read,
                    Err(e) => {
                        emit_error!(
                            variant.span(),
                            format!("Couldn't create binary parser for argument because: {e}")
                        );
                        continue;
                    }
                };
                ts.append_all(quote! { let #name = { #reading }; });
            }

            quote! {
                #ts
                ::std::result::Result::Ok(Cmd::#name #args_pattern)
            }
        };
        read_impl.append_all(quote! { #prefix => { #read }, });

        let ret = processed_attibutes.query.is_some();
        has_return.append_all(quote! { Cmd::#name #args_pattern => #ret, });

        'queries: {
            if let Some(query) = processed_attibutes.query {
                let text = match parsers::from_text(&query) {
                    Ok(text) => text,
                    Err(e) => {
                        emit_error!(
                            query.span(),
                            format!("Couldn't create text parser for query because: {e}")
                        );
                        break 'queries;
                    }
                };
                let bin = match parsers::read(&query) {
                    Ok(bin) => bin,
                    Err(e) => {
                        emit_error!(
                            query.span(),
                            format!("Couldn't create binary parser for query because: {e}")
                        );
                        break 'queries;
                    }
                };

                let bin_fn_name = format_ident!("parse_{name}_data_binary");
                let text_fn_name = format_ident!("parse_{name}_data_text");
                query_parsers.append_all(quote! {
                    pub fn #bin_fn_name(r: &mut impl ::std::io::Read) -> ::anyhow::Result<#query> {
                        ::std::result::Result::Ok(#bin)
                    }
                    pub fn #text_fn_name<'a>(args: &mut impl Iterator<Item = &'a str>) -> ::anyhow::Result<#query> {
                        ::std::result::Result::Ok(#text)
                    }
                });
            }
        }
    }

    let res = quote! {
        impl Parsable for #enum_ident {
            fn read_from_str<'a>(args: &mut impl Iterator<Item = &'a str>) -> ::anyhow::Result<Self> {
                let p = args.next().ok_or(::anyhow::Error::msg("Missing command prefix"))?;

                match p {
                    #from_text_impl
                    _ => ::std::result::Result::Err(::anyhow::Error::msg("Unknown command prefix"))
                }
            }
            fn write_str(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    #display_impl
                }
                Ok(())
            }

            fn read_binary(r: &mut impl ::std::io::Read) -> ::anyhow::Result<Self> {
                let mut prefix = [0u8; 1];
                ::std::io::Read::read_exact(r, &mut prefix[..1])
                    .map_err(|_| ::anyhow::Error::msg("Missing command prefix"))?;
                let prefix = prefix[0] as char;

                match prefix {
                    #read_impl
                    _ => ::std::result::Result::Err(::anyhow::Error::msg("Unknown command prefix"))
                }
            }
            fn write_binary(&self, w: &mut impl ::std::io::Write) -> ::anyhow::Result<()> {
                #[allow(clippy::char_lit_as_u8)]
                match self {
                    #write_impl
                }
                w.flush()?;
                Ok(())
            }
        }

        impl #enum_ident {
            pub fn has_return(&self) -> bool {
                match self {
                    #has_return
                }
            }
            #query_parsers
        }
    };

    proc_macro::TokenStream::from(res)
}
