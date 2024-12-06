use proc_macro::TokenStream;
use proc_macro2 as pm2;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, Data::Struct, DeriveInput, Error, Field,
    Fields::Named, Ident, Result, Token, Type,
};

type PunctuatedTStream = Punctuated<pm2::TokenStream, Token![,]>;

struct FieldMeta {
    iden: Ident,
    builder_iden: Ident,
    typ: Type,
}

fn field_meta(field: &Field) -> FieldMeta {
    let fid = field.ident.as_ref().unwrap();
    let bfid = format_ident!("b_{}", fid);
    let t = field.ty.to_owned();
    FieldMeta {
        iden: fid.to_owned(),
        builder_iden: bfid,
        typ: t,
    }
}

fn struct_fields(derive: &DeriveInput) -> Result<Vec<Field>> {
    let fields = match &derive.data {
        Struct(s) => match &s.fields {
            Named(fs) => Ok(&fs.named),
            _ => Err(Error::new_spanned(
                derive,
                "Un-named fields, cannot create builder.",
            )),
        },
        _ => Err(Error::new_spanned(derive, "Not a struct.")),
    }?;
    Ok(fields.iter().map(|f| f.to_owned()).collect())
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let some = quote! { std::option::Option::Some };
    let none = quote! { std::option::Option::None };
    let option = quote! { std::option::Option };
    let input = parse_macro_input!(input as DeriveInput);
    let fs = struct_fields(&input).unwrap();
    let fmlist = fs.iter().map(field_meta).collect::<Vec<_>>();
    let field_decl = fmlist
        .iter()
        .map(
            |FieldMeta {
                 builder_iden, typ, ..
             }| quote! { #builder_iden : #option<#typ> },
        )
        .collect::<PunctuatedTStream>();
    let builder_args = fmlist
        .iter()
        .map(|FieldMeta { builder_iden, .. }| quote! { #builder_iden : #none })
        .collect::<PunctuatedTStream>();
    let setters = fmlist
        .iter()
        .map(
            |FieldMeta {
                 iden,
                 builder_iden,
                 typ,
             }| {
                quote! {
                    pub fn #iden(&mut self, arg: #typ) -> &mut Self {
                        self.#builder_iden = #some(arg);
                        self
                    }
                }
            },
        )
        .collect::<pm2::TokenStream>();
    let args = fmlist
        .iter()
        .map(
            |FieldMeta {
                 iden, builder_iden, ..
             }| {
                quote! { #iden : std::mem::take(&mut self.#builder_iden)? }
            },
        )
        .collect::<PunctuatedTStream>();

    let id = input.ident;
    let bid = format_ident!("{}Builder", id);
    let exprs = quote! {
        struct #bid { #field_decl }

        impl #bid {
            #setters

            pub fn build(&mut self) -> #option<#id> {
                #some(#id { #args })
            }
        }

        impl #id {
            pub fn builder() -> #bid {
                #bid { #builder_args }
            }
        }
    };

    let t = TokenStream::from(exprs);
    t
}
