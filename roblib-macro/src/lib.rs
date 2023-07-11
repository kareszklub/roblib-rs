extern crate proc_macro;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Command)]
pub fn derive_command(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inp = parse_macro_input!(item as DeriveInput);
    let ident = &inp.ident;

    let res = quote! {
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

#[proc_macro_derive(Event)]
pub fn derive_event(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inp = parse_macro_input!(item as DeriveInput);
    let ident = &inp.ident;

    let res = quote! {
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
