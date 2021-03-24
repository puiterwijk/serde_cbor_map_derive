use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

use crate::{parser_helper, FieldType};

pub(crate) fn impl_derive_serialize_int_map(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => panic!("Serialize_int_map applied to invalid struct type"),
        },
        _ => panic!("Serialize_int_map applied to invalid type"),
    };

    let quotes_list = fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().expect("No identifier");
            let (_, is_optional) = parser_helper::get_field_type_and_optionality(field);

            let (attr_key, field_type) =
                match parser_helper::get_field_matcher_and_catchall_type(field) {
                    Some(val) => val,
                    None => return (quote! {}, quote! {}),
                };

            let attr_counter = if field_type.is_wildcard() {
                quote! { + self.#ident.num_items() }
            } else if field_type.is_phantom() {
                quote! {}
            } else {
                /* A field that's not catchall, has 1 value */
                quote! { + 1 }
            };

            let attr_serializer = match field_type {
                FieldType::Normal => {
                    if is_optional {
                        quote! {
                            if let Some(value) = &self.#ident {
                                map.serialize_entry(&#attr_key, &value)?;
                            }
                        }
                    } else {
                        quote! {
                            map.serialize_entry(&#attr_key, &self.#ident)?;
                        }
                    }
                }
                FieldType::WildCardFields => {
                    panic!("TODO: CatchallType::Fields implementation");
                }
                FieldType::WildCardUnknown => {
                    quote! {
                        for (k, v) in self.#ident.iter() {
                            map.serialize_entry(k, v)?;
                        }
                    }
                }
                FieldType::Phantom => quote! {},
            };

            (attr_counter, attr_serializer)
        })
        .collect::<Vec<(_, _)>>();
    let (attr_counters, attr_serializers) = crate::utils::list_to_tuple_2(quotes_list);

    let res = TokenStream::from(quote! {
        impl #impl_generics serde::Serialize for #ident #ty_generics #where_clause {
            fn serialize<SERIALIZER_TYPE>(&self, serializer: SERIALIZER_TYPE) -> core::result::Result<SERIALIZER_TYPE::Ok, SERIALIZER_TYPE::Error>
            where
                SERIALIZER_TYPE: serde::Serializer,
            {
                use serde::ser::SerializeMap;
                use serde_int_map::UnknownKeyHandler;

                let count = 0
                    #(#attr_counters)*
                ;

                let mut map = serializer.serialize_map(Some(count))?;
                #(#attr_serializers)*
                map.end()
            }
        }
    });

    #[cfg(feature = "print_tokenstreams")]
    println!("Serialization tokenstream for {}: {}", ident, res);

    res
}
