use proc_macro::{TokenStream};
use proc_macro2::{Ident};
use syn::{Attribute, ItemStruct};
use syn::{parse_macro_input, ExprArray};
use quote::{quote, format_ident};
use syn::Expr;
use syn::Token;
use syn::bracketed;
use syn::punctuated::Punctuated;

#[proc_macro_derive(Segment, attributes(depends_on))]
pub fn derive_segment(input: TokenStream) -> TokenStream {
    let ItemStruct { ident, fields, .. } = parse_macro_input!(input);
    let binary_converter = quote! { BinaryConverter };
    let cursor = quote! { std::io::Cursor };

    let field_names = fields.iter().map(|f| {
        f.ident.clone()
    }).collect::<Vec<Option<Ident>>>();

    let field_values = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        quote! { self.#field_name }
    }).collect::<Vec<_>>();

    let mut depends_on = vec![];

    for field in fields.iter() {
        let ident = field.ident.as_ref().unwrap().to_string();

        if field.attrs.iter().any(|attr| attr.path().is_ident("depends_on")) {
            depends_on.push(Some(ident));
        }
    }

    let initializers = fields.iter().map(|f| {
        quote! { #binary_converter::read_from(&mut reader, vec![]).unwrap() }
    });

    let output = quote! {
        impl #ident {
            pub fn from_binary(data: Vec<u8>) -> Self {
                let mut reader = #cursor::new(data);

                let mut instance = Self {
                    #(#field_names: #initializers),*
                };

                instance
            }

            pub fn test(&mut self) {
                println!("{:?}", vec![#(&#depends_on),*]);
            }
        }
    };

    TokenStream::from(output)
}