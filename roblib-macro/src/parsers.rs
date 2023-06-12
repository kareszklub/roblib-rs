use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse2, spanned::Spanned, Type};

static PRIMITIVES: [&str; 12] = [
    "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64",
];

pub fn from_text(t: &Type) -> Result<TokenStream, String> {
    let mut get_next_arg = false;

    let text = match t {
        Type::Array(a) => {
            let parse_text = from_text(&a.elem)?;

            let mut text = TokenStream::new();

            let len = parse2::<syn::LitInt>(a.len.to_token_stream())
                .map_err(|_| "Couldn't parse array length")?
                .base10_parse::<usize>()
                .map_err(|_| "Couldn't parse array length")?;

            for _ in 0..len {
                text.append_all(quote! { {#parse_text}, });
            }

            quote! {{ [#text] }}
        }

        Type::Path(p) => 'hack: {
            get_next_arg = true;

            if let Some(id) = p.path.get_ident() {
                if PRIMITIVES.iter().any(|t| id == t) {
                    let err = format!("Couldn't parse {id}");
                    break 'hack quote! {{
                        s.parse::<#id>()
                            .map_err(|_| ::anyhow::Error::msg(#err))?
                    }};
                }

                if id == "bool" {
                    break 'hack quote! {{
                        s.parse::<u8>()
                            .map_err(|_| ::anyhow::Error::msg("Couldn't parse bool"))? != 0
                    }};
                }

                if id == "String" {
                    break 'hack quote! { { s.to_string() } };
                }
            }

            if let Some(l) = p.path.segments.last() {
                if l.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(generics) = &l.arguments {
                        if let Some(syn::GenericArgument::Type(t)) = generics.args.first() {
                            let parse = from_text(t)?;
                            break 'hack quote! {{
                                let is_some = s.parse::<u8>()
                                    .map_err(|_| ::anyhow::Error::msg("Couldn't parse bool"))? != 0;

                                if is_some {
                                    Some({ #parse })
                                } else {
                                    None
                                }
                            }};
                        }
                    }
                }
            }

            get_next_arg = false;
            quote! { { #p::read_from_str(args)? } }
        }

        Type::Tuple(t) => {
            let mut text = TokenStream::new();

            for e in &t.elems {
                let parse_text = from_text(e)?;
                text.append_all(quote! { {#parse_text}, });
            }

            quote! { {(#text)} }
        }

        Type::Paren(p) => from_text(&p.elem)?,

        t => {
            return Err(format!(
                "Unsupported type {}",
                t.span().source_text().unwrap_or_else(|| format!("{t:?}"))
            ))
        }
    };

    Ok(if get_next_arg {
        quote! {{
            let s = args.next().ok_or_else(|| ::anyhow::Error::msg("Missing argument"))?;
            #text
        }}
    } else {
        quote! {{ #text }}
    })
}

pub fn read(t: &Type) -> Result<TokenStream, String> {
    let bin = match t {
        Type::Array(a) => {
            let parse_bin = read(&a.elem)?;

            let mut bin = TokenStream::new();

            let len = parse2::<syn::LitInt>(a.len.to_token_stream())
                .map_err(|_| "Couldn't parse array length")?
                .base10_parse::<usize>()
                .map_err(|_| "Couldn't parse array length")?;

            for _ in 0..len {
                bin.append_all(quote! { #parse_bin, });
            }

            quote! { [#bin] }
        }

        Type::Path(p) => 'hack: {
            if let Some(id) = p.path.get_ident() {
                if PRIMITIVES.iter().any(|t| id == t) {
                    break 'hack quote! {
                        let mut buf = [0; ::std::mem::size_of::<#id>()];
                        r.read_exact(&mut buf)?;
                        #id ::from_be_bytes(buf)
                    };
                }

                if id == "bool" {
                    break 'hack quote! {
                        let mut buf = [0; ::std::mem::size_of::<u8>()];
                        r.read_exact(&mut buf)?;
                        u8::from_be_bytes(buf) != 0
                    };
                }

                if id == "String" {
                    break 'hack quote! {
                        let mut len = [0; ::std::mem::size_of::<u16>()];
                        r.read_exact(&mut len)?;
                        let len = u16::from_be_bytes(len);
                        let mut buf = vec![0; len as usize];
                        r.read_exact(&mut buf)?;
                        String::from_utf8(buf)
                            .map_err(|_| ::anyhow::Error::msg("String not valid utf8"))?
                    };
                }
            }

            if let Some(l) = p.path.segments.last() {
                if l.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(generics) = &l.arguments {
                        if let Some(syn::GenericArgument::Type(t)) = generics.args.first() {
                            let parse = read(t)?;
                            break 'hack quote! {{
                                let is_some = {
                                    let mut buf = [0; ::std::mem::size_of::<u8>()];
                                    r.read_exact(&mut buf)?;
                                    u8::from_be_bytes(buf) != 0
                                };

                                if is_some {
                                    Some({ #parse })
                                } else {
                                    None
                                }
                            }};
                        }
                    }
                }
            }

            quote! { { #p::read_binary(r)? } }
        }

        Type::Tuple(t) => {
            let mut bin = TokenStream::new();

            for e in &t.elems {
                let parse_bin = read(e)?;
                bin.append_all(quote! { #parse_bin, });
            }

            quote! { (#bin) }
        }

        Type::Paren(p) => read(&p.elem)?,

        t => {
            return Err(format!(
                "Unsupported type {}",
                t.span().source_text().unwrap_or_else(|| format!("{t:?}"))
            ))
        }
    };

    Ok(quote! {{ #bin }})
}

pub fn display(t: &Type) -> Result<TokenStream, String> {
    let text = match t {
        Type::Array(a) => {
            let display_text = display(&a.elem)?;
            quote! {
                for i in i.iter() {
                    #display_text
                }
            }
        }

        Type::Path(p) => 'hack: {
            if let Some(id) = p.path.get_ident() {
                if PRIMITIVES.iter().any(|p| id == p) || id == "String" {
                    break 'hack quote! {
                        write!(f, "{}", i)?;
                    };
                }

                if id == "bool" {
                    break 'hack quote! {
                        write!(f, "{}", if *i { "0" } else { "1" })?;
                    };
                }
            };

            if let Some(l) = p.path.segments.last() {
                if l.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(generics) = &l.arguments {
                        if let Some(syn::GenericArgument::Type(t)) = generics.args.first() {
                            let display = display(t)?;
                            break 'hack quote! {{
                                write!(f, "{}", i.is_some() as u8)?;
                                if let Some(i) = i {
                                    #display
                                }
                            }};
                        }
                    }
                }
            }

            quote! { write!(f, "{}", i)?; }
        }

        Type::Tuple(t) => {
            let mut disp = TokenStream::new();

            for (index, e) in t.elems.iter().enumerate() {
                let display = display(e)?;
                let index = syn::Index::from(index);
                disp.append_all(quote! { { let i = &i.#index; #display } });
            }

            quote! { #disp }
        }

        Type::Paren(p) => display(&p.elem)?,

        t => {
            return Err(format!(
                "Unsupported type {}",
                t.span().source_text().unwrap_or_else(|| format!("{t:?}"))
            ))
        }
    };

    Ok(text)
}

pub fn write(t: &Type) -> Result<TokenStream, String> {
    let bin = match t {
        Type::Array(a) => {
            let w = write(&a.elem)?;
            quote! {
                for i in i.iter() {
                    #w
                }
            }
        }

        Type::Path(p) => 'hack: {
            if let Some(id) = p.path.get_ident() {
                if PRIMITIVES.iter().any(|p| id == p) {
                    break 'hack quote! {
                        w.write_all(&i.to_be_bytes())?;
                    };
                }

                if id == "bool" {
                    break 'hack quote! {
                        w.write_all(&(*i as u8).to_be_bytes())?;
                    };
                }
                if id == "String" {
                    break 'hack quote! {
                        w.write_all(&(i.len() as u16).to_be_bytes())?;
                        w.write_all(i.as_bytes())?;
                    };
                }
            };

            if let Some(l) = p.path.segments.last() {
                if l.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(generics) = &l.arguments {
                        if let Some(syn::GenericArgument::Type(t)) = generics.args.first() {
                            let write = write(t)?;
                            break 'hack quote! {{
                                w.write_all(&(i.is_some() as u8).to_be_bytes())?;
                                if let Some(i) = i {
                                    #write
                                }
                            }};
                        }
                    }
                }
            }

            quote! { i.write_binary(w)?; }
        }

        Type::Tuple(t) => {
            let mut w = TokenStream::new();

            for (index, e) in t.elems.iter().enumerate() {
                let wr = write(e)?;
                let index = syn::Index::from(index);
                w.append_all(quote! { { let i = &i.#index; #wr } });
            }

            quote! { #w }
        }

        Type::Paren(p) => write(&p.elem)?,

        t => {
            return Err(format!(
                "Unsupported type {}",
                t.span().source_text().unwrap_or_else(|| format!("{t:?}"))
            ))
        }
    };

    Ok(bin)
}
