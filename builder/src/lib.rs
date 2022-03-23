use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, AngleBracketedGenericArguments, DeriveInput, Expr, Field,
    FieldValue, GenericArgument, Ident, ImplItemMethod, PathArguments, Type, TypePath, Visibility,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let ident = ast.ident;
    let builder_ident = format_ident!("{}Builder", ident);

    let original_fields: Vec<(Field, bool)> = match ast.data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .cloned()
                .map(|field| {
                    if let Type::Path(TypePath {
                        qself: None,
                        ref path,
                    }) = field.ty
                    {
                        if let (1, Some(segment)) = (path.segments.len(), path.segments.first()) {
                            if stringify!(Option) == format!("{}", segment.ident) {
                                if let PathArguments::AngleBracketed(
                                    AngleBracketedGenericArguments {
                                        colon2_token: None,
                                        ref args,
                                        ..
                                    },
                                ) = segment.arguments
                                {
                                    {
                                        if let (1, Some(GenericArgument::Type(ty))) =
                                            (args.len(), args.first())
                                        {
                                            return (
                                                Field {
                                                    attrs: field.attrs.clone(),
                                                    vis: field.vis,
                                                    ident: field.ident,
                                                    colon_token: field.colon_token,
                                                    ty: ty.clone(),
                                                },
                                                true,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    (field, false)
                })
                .collect(),
            _ => panic!("Can only deal with named fields atm"),
        },
        _ => panic!("Can only deal with structs atm"),
    };

    let builder_fields: Vec<Field> = original_fields
        .iter()
        .map(|(field, _)| {
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
        .map(|(field, _)| field.ident.clone().expect("named fields must have idents"))
        .collect();

    let method_decls: Vec<ImplItemMethod> = original_fields
        .iter()
        .map(|(field, _)| {
            let ident = &field.ident;
            let ty = &field.ty;
            parse_quote!(fn #ident(&mut self, #ident:#ty) -> &mut Self {
                self.#ident = Some(#ident);
                self
            })
        })
        .collect();

    let assign_to_type: Vec<FieldValue> = original_fields
        .iter()
        .map(|(field, optional)| {
            let ident = field
                .ident
                .clone()
                .expect("only works with named fields anyway");

            if *optional {
                parse_quote!(#ident: self.#ident.take() )
            } else {
                parse_quote!(#ident: unsafe { self.#ident.take().unwrap_unchecked() })
            }
        })
        .collect();

    let constructor: Expr = parse_quote!(Ok(#ident {
        #(#assign_to_type),*
    }));

    let assignment: Expr =
        original_fields
            .iter()
            .fold(constructor, |expression, (field, optional)| {
                if *optional {
                    expression
                } else {
                    let ident = field.ident.clone().expect("only works with named fields");
                    let warning = format!("{} not set", ident);
                    parse_quote!(
                        if self.#ident.is_some() {
                        #expression
                    } else {
                        Err(#warning.into())
                    }
                    )
                }
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
