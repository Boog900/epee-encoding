#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Data, DataStruct, DeriveInput, Expr, Field, GenericParam,
    Generics, Lit, Type,
};

#[proc_macro_derive(EpeeObject, attributes(epee_default, epee_alt_name))]
pub fn derive_epee_object(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (_impl_generics, _ty_generics, _where_clause) = generics.split_for_impl();

    let output = match input.data {
        Data::Struct(data) => derive_object(&data, &struct_name),
        _ => panic!("Only structs can be epee objects"),
    };

    output.into()
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(epee_encoding::EpeeValue));
        }
    }
    generics
}

fn derive_object(data: &DataStruct, struct_name: &Ident) -> TokenStream {
    let field_info = data
        .fields
        .iter()
        .map(|field| get_field_data(field))
        .collect::<Vec<_>>();
    let field_names = field_info
        .iter()
        .map(|field_info| field_info.0.clone())
        .collect::<Vec<_>>();
    let field_types = field_info
        .iter()
        .map(|field_info| field_info.1.clone())
        .collect::<Vec<_>>();
    let field_default_val = field_info
        .iter()
        .map(|field_info| field_info.2.clone())
        .collect::<Vec<_>>();
    let field_alt_names = field_info
        .iter()
        .map(|field_info| field_info.3.clone())
        .collect::<Vec<_>>();

    let encoded_field_names = field_names
        .iter()
        .zip(field_alt_names)
        .map(|(name, alt)| {
            if let Some(alt) = alt {
                match alt {
                    Lit::Str(name) => name.value(),
                    _ => panic!("Alt name was not a string"),
                }
            } else {
                name.to_string()
            }
        })
        .collect::<Vec<_>>();

    let builder_name = Ident::new(
        &format!("__{}EpeeBuilder", struct_name.to_string()),
        Span::call_site(),
    );

    let builder_struct = build_builder_struct(&builder_name, &field_names, &field_types);
    let default_builder_impl = build_default_impl(&builder_name, &field_names, &field_default_val);
    let builder_impl = build_builder_impl(
        &struct_name,
        &builder_name,
        &field_names,
        &encoded_field_names,
    );

    let mod_name = Ident::new(&format!("__epee{}", struct_name), Span::call_site());
    let object_impl = build_object_impl(
        &struct_name,
        &mod_name,
        &builder_name,
        &field_names,
        &field_default_val,
        &encoded_field_names,
    );

    quote! {
        mod #mod_name {
            use super::*;
            #builder_struct

            #default_builder_impl

            #builder_impl
        }

        #object_impl
    }
}

fn get_field_data(field: &Field) -> (Ident, &Type, Option<Expr>, Option<Lit>) {
    let ident = field.ident.clone().unwrap();
    let ty = &field.ty;
    // If this field has a default value find it
    let default_val: Option<Expr> = field
        .attrs
        .iter()
        .find(|f| f.path().is_ident("epee_default"))
        .map(|f| f.parse_args().unwrap());
    // If this field has a different name when encoded find it
    let alt_name: Option<Lit> = field
        .attrs
        .iter()
        .find(|f| f.path().is_ident("epee_alt_name"))
        .map(|f| f.parse_args().unwrap());

    (ident, ty, default_val, alt_name)
}

fn build_builder_struct(
    builder_name: &Ident,
    field_names: &[Ident],
    field_types: &[Type],
) -> TokenStream {
    quote! {
        pub struct #builder_name {
            #(#field_names: Option<#field_types>),*
        }
    }
}

fn build_default_impl(
    struct_name: &Ident,
    field_names: &[Ident],
    field_default_vals: &[Option<Expr>],
) -> TokenStream {
    let mut values = TokenStream::new();
    for (default_val, name) in field_default_vals.iter().zip(field_names) {
        if let Some(default_val) = default_val {
            values = quote! {
                #values
                #name: Some(#default_val),
            }
        } else {
            values = quote! {
                #values
                #name: None,
            }
        }
    }
    quote! {
        impl Default for #struct_name {
            fn default() -> Self {
                #struct_name {
                    #values
                }
            }
        }
    }
}

fn build_builder_impl(
    struct_name: &Ident,
    builder_name: &Ident,
    field_names: &[Ident],
    encoded_field_names: &[String],
) -> TokenStream {
    quote! {
        impl epee_encoding::EpeeObjectBuilder<#struct_name> for #builder_name {
            fn add_field<R: std_shims::io::Read>(&mut self, name: &str, r: &mut R) -> std_shims::io::Result<()> {
                match name {
                    #(#encoded_field_names => {let _ = self.#field_names.insert(epee_encoding::read_epee_value(r)?);},)*
                    _ => epee_encoding::skip_epee_value(r)?,
                };

                Ok(())
            }

            fn finish(self) -> std_shims::io::Result<#struct_name> {
                Ok(#struct_name {
                    #(#field_names: self.#field_names.ok_or_else(|| std_shims::io::Error::new(std_shims::io::ErrorKind::Other, "Required field was not found!"))?),*
                })
            }
        }
    }
}

fn build_object_impl(
    struct_name: &Ident,
    mod_name: &Ident,
    builder_name: &Ident,
    field_names: &[Ident],
    field_default_vals: &[Option<Expr>],
    encoded_field_names: &[String],
) -> TokenStream {
    let mut handle_defaults = TokenStream::new();

    let mut encode_fields = TokenStream::new();

    let numb_o_fields: u64 = field_names.len().try_into().unwrap();

    for ((field_name, encoded_field_name), default_val) in field_names
        .iter()
        .zip(encoded_field_names)
        .zip(field_default_vals)
    {
        if let Some(default_val) = default_val {
            handle_defaults = quote! {
                #handle_defaults
                if self.#field_name == #default_val {
                    numb_o_fields -= 1;
                };
            };
            encode_fields = quote! {
                #encode_fields
                if self.#field_name != #default_val {
                    epee_encoding::write_field(&self.#field_name, &#encoded_field_name, w)?;
                };
            }
        } else {
            encode_fields = quote! {
                #encode_fields
                epee_encoding::write_field(&self.#field_name, &#encoded_field_name, w)?;
            }
        }
    }

    quote! {
        impl EpeeObject for #struct_name {
            type Builder = #mod_name::#builder_name;

            fn write<W: std_shims::io::Write>(&self, w: &mut W) -> std_shims::io::Result<()> {
                let mut numb_o_fields: u64 = #numb_o_fields;

                #handle_defaults

                epee_encoding::varint::write_varint(numb_o_fields, w)?;

                #encode_fields

                Ok(())
            }
        }
    }
}
