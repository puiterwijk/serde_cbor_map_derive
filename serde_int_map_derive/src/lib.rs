use proc_macro::TokenStream;

mod de;
mod parser_helper;
mod ser;
mod utils;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum FieldType {
    Normal,
    WildCardFields,
    WildCardUnknown,
    Phantom,
}

impl FieldType {
    fn is_wildcard(self) -> bool {
        self == FieldType::WildCardUnknown || self == FieldType::WildCardFields
    }

    fn is_phantom(self) -> bool {
        self == FieldType::Phantom
    }
}

#[proc_macro_derive(
    Serialize_int_map,
    attributes(int_map_id, int_map_unknown, int_map_ignore, int_map_phantom)
)]
pub fn derive_serialize_int_map(input: TokenStream) -> TokenStream {
    ser::impl_derive_serialize_int_map(input)
}

#[proc_macro_derive(
    Deserialize_int_map,
    attributes(int_map_id, int_map_unknown, int_map_ignore, int_map_phantom)
)]
pub fn derive_deserialize_int_map(input: TokenStream) -> TokenStream {
    de::impl_derive_deserialize_int_map(input)
}
