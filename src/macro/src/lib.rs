use proc_macro::{TokenStream};
use std::collections::BTreeMap;
use proc_macro2::{Ident};
use syn::{Attribute, ItemStruct};
use syn::{parse_macro_input, ExprArray};
use quote::{quote, format_ident};
use quote::__private::ext::RepToTokensExt;
use syn::Expr;
use syn::Token;
use syn::bracketed;
use syn::punctuated::Punctuated;
use syn::Meta;
use syn::Lit;
use syn::parse::Parse;
use syn::parse::ParseStream;

struct MyAttr {
    pub name: Ident,
}

impl Parse for MyAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;

        Ok(Self { name })
    }
}

#[proc_macro_derive(Segment, attributes(depends_on))]
pub fn derive_segment(input: TokenStream) -> TokenStream {
    let ItemStruct { ident, fields, attrs, .. } = parse_macro_input!(input);
    let binary_converter = quote! { BinaryConverter };
    let to_bytes = quote! { ToBytes };
    let cursor = quote! { std::io::Cursor };

    let field_names = fields.iter().map(|f| {
        f.ident.clone()
    }).collect::<Vec<Option<Ident>>>();

    let field_values = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        quote! { self.#field_name }
    }).collect::<Vec<_>>();

    let mut depends_on: BTreeMap<Option<Ident>, Vec<Ident>> = BTreeMap::new();

    for field in fields.iter() {
        let ident = field.ident.clone();

        if field.attrs.iter().any(|attr| attr.path().is_ident("depends_on")) {

            let mut dependencies: Vec<Ident> = vec![];
            field.attrs.iter().for_each(|attr| {
                for a in attr.parse_args_with(Punctuated::<MyAttr, Token![,]>::parse_terminated).unwrap() {
                    dependencies.push(a.name);
                }
            });

            depends_on.insert(ident, dependencies);
        }
    }

    let initializers = fields.iter().map(|f| {
        let field_name = f.ident.clone();
        let field_type = f.ty.clone();

        if let Some(dependencies) = depends_on.get(&field_name) {
            quote! {
                {
                    let mut buffer: Vec<u8> = vec![];
                    #(
                        let bytes = #to_bytes::to_bytes(
                            &mut cache.#dependencies,
                        );
                        buffer.extend(bytes);
                    )*
                    #binary_converter::read_from(&mut reader, buffer).unwrap()
                }
            }
        } else {
            quote! {
                {
                    let value: #field_type = #binary_converter::read_from(&mut reader, vec![]).unwrap();
                    cache.#field_name = value.clone();
                    value
                }
            }
        }
    });

    let output = quote! {
        impl #ident {
            pub fn from_binary(data: Vec<u8>) -> Self {
                let mut cache = Self {
                    #(#field_names: Default::default()),*
                };

                let mut reader = #cursor::new(data);

                let mut instance = Self {
                    #(#field_names: #initializers),*
                };

                instance
            }

            pub fn test(&mut self) {
                // println!("{:?}", vec![#(&#depends_on),*]);
                // println!("{:?}", #valyes);
            }
        }
    };

    TokenStream::from(output)
}