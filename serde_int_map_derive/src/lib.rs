use proc_macro::TokenStream;

mod de;
mod parser_helper;
mod ser;
mod utils;

enum CatchallType {
    Fields,
    Unknown,
}

#[proc_macro_derive(Serialize_int_map, attributes(int_map_id, int_map_unknown))]
pub fn derive_serialize_int_map(input: TokenStream) -> TokenStream {
    ser::impl_derive_serialize_int_map(input)
}

#[proc_macro_derive(Deserialize_int_map, attributes(int_map_id, int_map_unknown))]
pub fn derive_deserialize_int_map(input: TokenStream) -> TokenStream {
    de::impl_derive_deserialize_int_map(input)
}
