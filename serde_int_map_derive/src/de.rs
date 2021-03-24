use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, GenericParam, Lifetime, LifetimeDef};

use crate::{parser_helper, FieldType};

pub(crate) fn impl_derive_deserialize_int_map(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut input_with_de = input.clone();

    let (impl_generics_with_de, _, _) = {
        let de = Lifetime::new("'deserializer_life", Span::call_site());
        let de = LifetimeDef::new(de);
        input_with_de.generics.params.push(de.into());
        input_with_de.generics.split_for_impl()
    };
    let ident = input.ident;
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();

    let generic_names: Vec<_> = input
        .generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Type(type_param) => {
                if type_param.colon_token.is_some() {
                    panic!("Unsupported type param: {:?}", type_param);
                }
                type_param.ident.clone()
            }
            other => panic!("Unsupported generic: {:?}", other),
        })
        .collect();

    let visitor_phantom_names: Vec<_> = generic_names
        .iter()
        .map(|name| {
            let phantom_ident = format_ident!("_phantom_{}", name);
            quote! {
                #phantom_ident: std::marker::PhantomData<#name>,
            }
        })
        .collect();
    let visitor_phantom_installers: Vec<_> = generic_names
        .iter()
        .map(|name| {
            let phantom_ident = format_ident!("_phantom_{}", name);
            quote! {
                    #phantom_ident: std::marker::PhantomData,
            }
        })
        .collect();

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
        impl #impl_generics_with_de serde::Deserialize<'deserializer_life> for #ident #ty_generics #where_clause {
            fn deserialize<DESERIALIZER_TYPE>(deserializer: DESERIALIZER_TYPE) -> core::result::Result<#ident #ty_generics, DESERIALIZER_TYPE::Error>
            where
                DESERIALIZER_TYPE: serde::Deserializer<'deserializer_life>,
                S: PayloadState,
            {
                use serde_int_map::UnknownKeyHandler;

                struct OurVisitor #ty_generics {
                    #(#visitor_phantom_names)*
                };

                impl #impl_generics_with_de serde::de::Visitor<'deserializer_life> for OurVisitor #ty_generics #where_clause {
                    type Value = #ident #ty_generics;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(formatter, "a map")
                    }

                    fn visit_map<V>(self, mut map: V) -> core::result::Result<#ident #ty_generics, V::Error>
                    where
                        V: serde::de::MapAccess<'deserializer_life>,
                    {
                        #(#attr_placeholders)*

                        while let Some(_int_map_key) = map.next_key::<i64>()? {
                            match _int_map_key {
                                #(#attr_matchers)*
                            }
                        }

                        Ok(#ident {
                            #(#attr_installers)*
                        })
                    }
                }

                deserializer.deserialize_map(OurVisitor {
                    #(#visitor_phantom_installers)*
                })
            }
        }
    });

    #[cfg(feature = "print_tokenstreams")]
    println!("Deserialization tokenstream for {}: {}", ident, res);

    res
}
