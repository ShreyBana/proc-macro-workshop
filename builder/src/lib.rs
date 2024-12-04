use proc_macro::TokenStream;
use proc_macro2 as pm2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, punctuated::{self, Punctuated}, Data::Struct, DeriveInput, Error, Field, Fields::Named, Result, Token};

fn into_builder_field(field: &Field) -> pm2::TokenStream {
    let vis = field.vis.to_owned();
    let id = format_ident!("b_{}", field.ident.to_owned().unwrap());
    let t = field.ty.to_owned();
    quote! {
        #vis #id : Option<#t>
    }.into()
}

fn create_fields(derive: &DeriveInput) -> Result<Punctuated<pm2::TokenStream, Token![,]>> {
    let fields = match derive.data {
        Struct(ref ds) => match &ds.fields {
            Named(fs) => Ok(fs.named.to_owned()),
            _ => Err(Error::new_spanned(derive, "Un-named fields, cannot create builder."))
        },
        _ => Err(Error::new_spanned(derive, "Not a struct."))
    }?;
    let i = fields.iter()
        .map(into_builder_field);
    Ok(Punctuated::from_iter(i))
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let fields = create_fields(&input).unwrap();
    let id = input.ident;
    let bid = format_ident!("{}Builder", id);
    let const_args: Punctuated<Option<pm2::TokenStream>, Token![,]> =
                               Punctuated::from_iter(fields.iter().map(|_| None));

    let exprs = quote! {
        struct #bid { #fields }

        impl #id {
            pub fn builder() -> #bid {
                #bid { #const_args }
            }
        }
    };

    let t = TokenStream::from(exprs);
    eprintln!("TOKENS: {}", t);

    t
}
