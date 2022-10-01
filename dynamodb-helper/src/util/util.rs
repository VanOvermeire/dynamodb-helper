use quote::__private::Ident;
use syn::{Field, Type};
use syn::punctuated::{Punctuated};
use syn::token::Comma;

pub fn get_ident_and_type_of_field_annotated_with<'a>(fields: &'a Punctuated<Field, Comma>, name: &'a str) -> Option<(&'a Ident, &'a Type)> {
    fields.iter()
        .filter(|f| get_attribute(f, name).is_some())
        .map(|f| (f.ident.as_ref().unwrap(), &f.ty))
        .next()
}

fn get_attribute<'a>(f: &'a syn::Field, name: &'a str) -> Option<&'a syn::Attribute> {
    for attr in &f.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == name.to_string() {
            return Some(attr);
        }
    }
    None
}

pub fn get_relevant_field_info<'a>(f: &'a Field) -> (&'a Ident, String, &Type) {
    let name = &f.ident.as_ref().unwrap();
    let name_as_string = name.to_string();
    let field_type = &f.ty;
    (name, name_as_string, field_type)
}

pub fn matches_any_type<'a>(ty: &'a syn::Type, type_names: Vec<&str>) -> bool {
    type_names.iter().any(|v| matches_type(ty, v))
}

pub fn matches_type<'a>(ty: &'a syn::Type, type_name: &str) -> bool {
    if let syn::Type::Path(ref p) = ty {
        return p.path.segments[0].ident.to_string() == type_name.to_string()
    }
    false
}
