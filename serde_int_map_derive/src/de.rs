use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

use crate::{parser_helper, FieldType};

pub(crate) fn impl_derive_deserialize_int_map(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => panic!("Deserialize_Int_Map applied to invalid struct type"),
        },
        _ => panic!("Deserialize_Int_Map applied to invalid type"),
    };

    // Build the components based on attributes
    let quotes_list = fields.iter().map(|field| {
        let ident = field.ident.as_ref().expect("No identifier");
        let (non_option_ty, is_optional) = parser_helper::get_field_type_and_optionality(field);

        let (matcher, field_type) = match parser_helper::get_field_matcher_and_catchall_type(field) {
            Some(val) => val,
            // Ignore this field entirely
            None => return (
                quote!{},
                quote!{},
                quote!{},
                quote!{},
            ),
        };

        let attr_placeholder = match field_type {
            FieldType::Normal => quote! {
                let mut #ident: core::option::Option<#non_option_ty> = None;
            },
            FieldType::WildCardFields | FieldType::WildCardUnknown =>
            quote! {
                let mut #ident: #non_option_ty = #non_option_ty::new();
            },
            FieldType::Phantom => quote!{}
        };

        let attr_matcher = match field_type {
            FieldType::Normal => quote! {
                #matcher => {
                    if #ident.is_some() {
                        return Err(serde::de::Error::duplicate_field(stringify!(#ident)));
                    }
                    #ident = Some(map.next_value()?);
                }
            },
            FieldType::WildCardFields | FieldType::WildCardUnknown | FieldType::Phantom => quote!{},
        };

        let catchall_attr_matcher = match field_type {
            FieldType::Normal | FieldType::Phantom => quote! {},
            FieldType::WildCardFields => {
                panic!("TODO: CatchallType::Fields implementation");
            }
            FieldType::WildCardUnknown => {
                quote! {
                    if #ident.handles_key(catchall) {
                        #ident.fill_value(catchall, map.next_value()?);
                        continue;
                    }
                }
            }
        };

        let attr_installer = if is_optional || field_type.is_wildcard() {
            quote! {
                #ident,
            }
        } else if field_type.is_phantom() {
            quote! {
                #ident: std::marker::PhantomData,
            }
        } else {
            quote! {
                #ident: #ident.ok_or_else(|| serde::de::Error::missing_field(stringify!(#ident)))?,
            }
        };

        (attr_placeholder, attr_matcher, catchall_attr_matcher, attr_installer)
    })
    .collect::<Vec<(_, _, _, _)>>();
    let (attr_placeholders, attr_matchers, catchall_attr_matchers, attr_installers) =
        crate::utils::list_to_tuple_4(quotes_list);

    let mut attr_matchers = attr_matchers;
    // Add the catchall entry
    attr_matchers.push(quote! {
        catchall => {
            #(#catchall_attr_matchers)*

            // If none of the catchall-ers parsed it, it's unknown
            return Err(serde::de::Error::custom(format!("Field found with unknown field ID: {}", catchall)));
        },
    });
    let attr_matchers = attr_matchers;

    let res = TokenStream::from(quote! {
        impl<'de> serde::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde_int_map::UnknownKeyHandler;

                struct OurVisitor;

                impl<'de> serde::de::Visitor<'de> for OurVisitor {
                    type Value = #ident;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(formatter, "a map")
                    }

                    fn visit_map<V>(self, mut map: V) -> core::result::Result<#ident, V::Error>
                    where
                        V: serde::de::MapAccess<'de>,
                    {
                        #(#attr_placeholders)*

                        while let Some(_int_map_key) = map.next_key::<u32>()? {
                            match _int_map_key {
                                #(#attr_matchers)*
                            }
                        }

                        Ok(#ident {
                            #(#attr_installers)*
                        })
                    }
                }

                deserializer.deserialize_map(OurVisitor)
            }
        }
    });

    #[cfg(feature = "print_tokenstreams")]
    println!("Deserialization tokenstream for {}: {}", ident, res);

    res
}
