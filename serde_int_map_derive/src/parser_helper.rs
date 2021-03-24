use proc_macro::{TokenStream, TokenTree};
use syn::{Attribute, Field, GenericArgument, Path, PathArguments, Type, TypePath};

use crate::FieldType;

pub(crate) fn get_field_matcher_and_catchall_type(field: &Field) -> Option<(i64, FieldType)> {
    let int_map_id_attrs = field
        .attrs
        .iter()
        .filter(|attr| {
            let ident = attr.path.segments.first().unwrap().ident.to_string();
            ident == "int_map_id"
                || ident == "int_map_unknown"
                || ident == "int_map_ignore"
                || ident == "int_map_phantom"
        })
        .collect::<Vec<&Attribute>>();
    if int_map_id_attrs.is_empty() {
        panic!(format!(
            "No int_map_* found on field {}",
            field.ident.as_ref().unwrap()
        ));
    }
    if int_map_id_attrs.len() > 1 {
        panic!(format!(
            "Multiple int_map_* attributes found on field {}",
            field.ident.as_ref().unwrap()
        ));
    }
    let int_map_id_attr = int_map_id_attrs.first().unwrap();

    let field_type = match &int_map_id_attr
        .path
        .segments
        .first()
        .unwrap()
        .ident
        .to_string()[..]
    {
        "int_map_id" => FieldType::Normal,
        "int_map_fields" => FieldType::WildCardFields,
        "int_map_unknown" => FieldType::WildCardUnknown,
        "int_map_phantom" => FieldType::Phantom,
        "int_map_ignore" => return None,
        _ => panic!("Impossible"),
    };
    let matcher = match field_type {
        FieldType::WildCardFields | FieldType::WildCardUnknown | FieldType::Phantom => {
            // This can be any value, it's ignored anyway
            42
        }
        FieldType::Normal => {
            let token_stream = int_map_id_attr.tokens.clone();
            let token_stream = TokenStream::from(token_stream);
            let token = token_stream.into_iter().next().expect(&format!(
                "No int_map_id value on field {}",
                field.ident.as_ref().unwrap().to_string()
            ));
            let mut token_stream = match token {
                TokenTree::Group(group) => group,
                _ => panic!("Invalid token matched"),
            }
            .stream()
            .into_iter();
            let token = token_stream.next().expect(&format!(
                "Nothing found in parenthesis group for field {}",
                field.ident.as_ref().unwrap().to_string()
            ));
            let (plusmin, token) = match token {
                TokenTree::Punct(punct) => {
                    let next_token = token_stream.next().expect(&format!(
                        "Nothing found after punctuation character for field {}",
                        field.ident.as_ref().unwrap().to_string()
                    ));
                    (punct.as_char(), next_token)
                }
                other => ('+', other),
            };
            let token = match token {
                TokenTree::Literal(literal) => format!("{}{}", plusmin, literal.to_string()),
                _ => panic!(format!(
                    "Non-literal int_map_id value for field {}: {}, value: {:?}",
                    field.ident.as_ref().unwrap().to_string(),
                    token.to_string(),
                    token
                )),
            };
            token.parse::<i64>().expect(&format!(
                "Non-integer int_map_id value for field {}: {}",
                field.ident.as_ref().unwrap().to_string(),
                token.to_string()
            ))
        }
    };

    Some((matcher, field_type))
}

pub(crate) fn get_field_type_and_optionality(field: &Field) -> (Type, bool) {
    match &field.ty {
        Type::Path(TypePath {
            qself: None,
            path:
                Path {
                    leading_colon: _,
                    segments: seg,
                },
        }) => {
            let last_seg = &seg.last().expect("No last segment");
            match &last_seg.ident.to_string()[..] {
                "Option" => match &last_seg.arguments {
                    PathArguments::AngleBracketed(args) => {
                        match &args.args.first().expect("No argument to Option") {
                            GenericArgument::Type(ty) => match ty {
                                Type::Path(tp) => (Type::Path(tp.clone()), true),
                                _ => panic!("Non-path Option type"),
                            },
                            _ => panic!("Non-type Option argument"),
                        }
                    }
                    _ => panic!("Non-bracketed option"),
                },
                _ => (field.ty.clone(), false),
            }
        }
        _ => panic!("Unsupported type encontered"),
    }
}
