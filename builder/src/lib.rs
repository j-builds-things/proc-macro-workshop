use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, DeriveInput, Field, Visibility};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let ident = ast.ident;
    let builder_ident = format_ident!("{}Builder", ident);

    let builder_fields: Vec<Field> = match ast.data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .map(|field| {
                    let ty = field.ty.clone();
                    Field {
                        attrs: vec![],
                        vis: Visibility::Inherited,
                        ident: field.ident.clone(),
                        colon_token: field.colon_token,
                        ty: parse_quote!(::core::option::Option<#ty>),
                    }
                })
                .collect(),
            _ => panic!("Can only deal with named fields atm"),
        },
        _ => panic!("Can only deal with structs atm"),
    };

    let idents = builder_fields
        .iter()
        .map(|field| field.ident.clone().expect("named fields must have idents"));

    quote! {
       impl #ident {
          fn builder() -> #builder_ident {
                #builder_ident { #(#idents: None),* }
          }
       }
       struct #builder_ident {
          #(#builder_fields),*
       }
    }
    .into()
}
