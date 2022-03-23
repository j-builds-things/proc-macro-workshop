use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, DeriveInput, Expr, Field, FieldValue, Ident, ImplItemMethod,
    Visibility,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let ident = ast.ident;
    let builder_ident = format_ident!("{}Builder", ident);

    let original_fields: Vec<Field> = match ast.data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(fields) => fields.named.iter().cloned().collect(),
            _ => panic!("Can only deal with named fields atm"),
        },
        _ => panic!("Can only deal with structs atm"),
    };

    let builder_fields: Vec<Field> = original_fields
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
        .collect();

    let idents: Vec<Ident> = original_fields
        .iter()
        .map(|field| field.ident.clone().expect("named fields must have idents"))
        .collect();

    let method_decls: Vec<ImplItemMethod> = original_fields
        .iter()
        .map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            parse_quote!(fn #ident(&mut self, #ident:#ty) -> &mut Self {
                self.#ident = Some(#ident);
                self
            })
        })
        .collect();

    let assign_to_type: Vec<FieldValue> = idents
        .iter()
        .map(|ident| parse_quote!(#ident: unsafe { self.#ident.take().unwrap_unchecked() }))
        .collect();
    let constructor: Expr = parse_quote!(Ok(#ident {
        #(#assign_to_type),*
    }));

    let assignment: Expr = idents.iter().fold(constructor, |expression, ident| {
        let warning = format!("{} not set", ident);
        parse_quote!(
        if self.#ident.is_some() {
            #expression
        } else {
            Err(#warning.into())
        }
        )
    });

    quote! {
       impl #ident {
          fn builder() -> #builder_ident {
                   #builder_ident { #(#idents: None),* }
          }
       }
       struct #builder_ident {
          #(#builder_fields),*
       }

       impl #builder_ident {
          #(#method_decls)*

         pub fn build(&mut self) -> ::std::result::Result<#ident, ::std::boxed::Box<dyn ::std::error::Error>> {
             #assignment
         }
       }
    }
    .into()
}
