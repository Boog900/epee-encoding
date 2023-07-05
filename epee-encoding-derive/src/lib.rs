#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::ToString;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Expr, Fields, GenericParam, Generics, Lit,
};

#[proc_macro_derive(EpeeObject, attributes(epee_default, epee_alt_name, epee_flatten))]
pub fn derive_epee_object(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (_impl_generics, _ty_generics, _where_clause) = generics.split_for_impl();

    let output = match input.data {
        Data::Struct(data) => build(&data.fields, &struct_name),
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

fn build(fields: &Fields, struct_name: &Ident) -> TokenStream {
    let mut struct_fields = TokenStream::new();
    let mut default_values = TokenStream::new();
    let mut count_fields = TokenStream::new();
    let mut write_fields = TokenStream::new();

    let mut read_match_body = TokenStream::new();
    let mut read_catch_all = TokenStream::new();

    let mut object_finish = TokenStream::new();

    let numb_o_fields: u64 = fields.len().try_into().unwrap();

    for field in fields {
        let field_name = field.ident.clone().expect("Epee only accepts named fields");
        let field_type = &field.ty;
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

        let is_flattened = field
            .attrs
            .iter()
            .any(|f| f.path().is_ident("epee_flatten"));

        // Gets this objects epee name, the name its encoded with
        let epee_name = if let Some(alt) = alt_name {
            if is_flattened {
                panic!("Cant rename a flattened field")
            }
            match alt {
                Lit::Str(name) => name.value(),
                _ => panic!("Alt name was not a string"),
            }
        } else {
            field_name.to_string()
        };

        // This is fields part of a struct:
        // struct T {
        //  #struct_fields
        // }
        if is_flattened {
            struct_fields = quote! {
                #struct_fields
                #field_name: <#field_type as epee_encoding::EpeeObject>::Builder,
            };

            count_fields = quote! {
                #count_fields
                // This filed has been flattened so dont count it.
                numb_o_fields -= 1;
                // Add the flattend fields to this one.
                numb_o_fields += self.#field_name.number_of_fields();

            };

        } else {
            struct_fields = quote! {
                #struct_fields
                #field_name: Option<#field_type>,
            };
        }

        // `default_val`: this is the body of a default function:
        // fn default() -> Self {
        //    Self {
        //       #default_values
        //    }
        // }

        // `count_fields`: this is the part of the write function that takes
        // away from the number of fields if the field is the default value.

        // `write_fields`: this is the part of the write function that writes
        // this specific epee field.
        if let Some(default_val) = default_val {
            if is_flattened {
                panic!("Cant have a default on a flattened field");
            };

            default_values = quote! {
                #default_values
                #field_name: Some(#default_val),
            };

            count_fields = quote! {
                #count_fields
                if self.#field_name == #default_val {
                    numb_o_fields -= 1;
                };
            };

            write_fields = quote! {
                #write_fields
                if self.#field_name != #default_val {
                    epee_encoding::write_field(&self.#field_name, &#epee_name, w)?;
                }
            }
        } else {
            if !is_flattened {
                default_values = quote! {
                    #default_values
                    #field_name: None,
                };

                write_fields = quote! {
                    #write_fields
                    epee_encoding::write_field(&self.#field_name, #epee_name, w)?;
                };
            } else {
                default_values = quote! {
                    #default_values
                    #field_name: Default::default(),
                };

                write_fields = quote! {
                    #write_fields
                    self.#field_name.write_fields(w)?;
                };
            }
        };

        // This is what these values do:
        // fn add_field(name: &str, r: &mut r) -> Result<bool> {
        //    match name {
        //        #read_match_body
        //        _ => {
        //           #read_catch_all
        //           return Ok(false);
        //         }
        //    }
        //    Ok(true)
        // }
        if is_flattened {
            read_catch_all = quote! {
                #read_catch_all
                if self.#field_name.add_field(name, r)? {
                    return Ok(true);
                };
            };

            object_finish = quote! {
                #object_finish
                #field_name: self.#field_name.finish()?,
            };
        } else {
            read_match_body = quote! {
                #read_match_body
                #epee_name => {self.#field_name = Some(epee_encoding::read_epee_value(r)?);},
            };

            object_finish = quote! {
                #object_finish
                #field_name: self.#field_name.ok_or_else(|| epee_encoding::error::Error::Format("Required field was not found!"))?,
            };
        }
    }

    let builder_name = Ident::new(&format!("__{}EpeeBuilder", struct_name), Span::call_site());
    let mod_name = Ident::new(&format!("__{}_epee_module", struct_name), Span::call_site());

    let builder_impl = quote! {
        pub struct #builder_name {
            #struct_fields
        }

        impl Default for #builder_name {
            fn default() -> Self {
                Self {
                    #default_values
                }
            }
        }

        impl epee_encoding::EpeeObjectBuilder<#struct_name> for #builder_name {
            fn add_field<R: epee_encoding::io::Read>(&mut self, name: &str, r: &mut R) -> epee_encoding::error::Result<bool> {
                match name {
                    #read_match_body
                    _ => {
                        #read_catch_all
                        return Ok(false);
                    }
                };

                Ok(true)
            }

            fn finish(self) -> epee_encoding::error::Result<#struct_name> {
                Ok(#struct_name {
                    #object_finish
                })
            }
        }
    };

    let object_impl = quote! {
        impl EpeeObject for #struct_name {
            type Builder = #mod_name::#builder_name;

            fn number_of_fields(&self) -> u64 {
                let mut numb_o_fields: u64 = #numb_o_fields;
                #count_fields
                numb_o_fields
            }


            fn write_fields<W: epee_encoding::io::Write>(&self, w: &mut W) -> epee_encoding::error::Result<()> {

                #write_fields

                Ok(())
            }
        }
    };

    quote! {
        mod #mod_name {
            use super::*;
            #builder_impl
        }

        #object_impl
    }
}
