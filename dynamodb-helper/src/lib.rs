extern crate core;

mod util;

use crate::util::*;
use quote::quote;
use proc_macro::{TokenStream};
use quote::__private::Ident;
use syn::{parse_macro_input, DeriveInput};
use syn::Data::Struct;
use syn::DataStruct;
use syn::Fields::Named;
use syn::FieldsNamed;

#[proc_macro_derive(DynamoDb, attributes(partition,range,exclusion))]
pub fn create_dynamodb_helper(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let name = ast.ident;
    let helper_name = format!("{}Db", name);
    let helper_ident = Ident::new(&helper_name, name.span());


    let fields = match ast.data {
        Struct(DataStruct { fields: Named(FieldsNamed { ref named, .. }), .. }) => named,
        _ => unimplemented!("Only works for structs"),
    };

    let exclusion_list = get_macro_attribute(&ast.attrs, "exclusion");

    let partition_key_ident_and_type = get_ident_and_type_of_field_annotated_with(fields, "partition").expect("Partition key should be defined (with attribute #[partition])");
    let range_key_ident_and_type = get_ident_and_type_of_field_annotated_with(fields, "range");

    let from_struct_for_hashmap = build_from_struct_for_hashmap(&name, fields);
    let from_hashmap_for_struct = build_from_hashmap_for_struct(&name, fields);

    let new = new_method(&helper_ident);
    let build = build_method(&helper_ident);

    let create_table = tokenstream_or_empty_if_exclusion(
        create_table_method(partition_key_ident_and_type, range_key_ident_and_type), "create_table", &exclusion_list
    );
    let delete_table = tokenstream_or_empty_if_exclusion(
        delete_table_method(), "delete_table", &exclusion_list
    );

    let gets = get_methods(&name, partition_key_ident_and_type, range_key_ident_and_type);
    let batch_get = batch_get(&name, partition_key_ident_and_type, range_key_ident_and_type);

    let put = tokenstream_or_empty_if_exclusion(
        put_method(&name), "put", &exclusion_list,
    );
    let batch_put = tokenstream_or_empty_if_exclusion(
        batch_put_method(&name), "batch_put", &exclusion_list,
    );
    let delete = tokenstream_or_empty_if_exclusion(
        delete_method(&name, partition_key_ident_and_type, range_key_ident_and_type), "delete", &exclusion_list,
    );
    let scan = tokenstream_or_empty_if_exclusion(
        scan_method(&name), "scan", &exclusion_list,
    );

    let public_version = quote! {
        #from_struct_for_hashmap
        #from_hashmap_for_struct

        pub struct #helper_ident {
            pub client: aws_sdk_dynamodb::Client,
            pub table: String,
        }

        impl #helper_ident {
            #new
            #build

            #create_table
            #delete_table

            #put
            #gets
            #batch_get
            #batch_put
            #delete
            #scan
        }
    };

    public_version.into()
}

// fn tokenstream_or_empty_if_exclusion(stream: TokenStream2, method_name: &str, exclusions: &Vec<String>) -> TokenStream2 {
//     if exclusions.contains(&method_name.to_string()) {
//         quote! {}
//     } else {
//         stream
//     }
// }

// fn get_macro_attribute(attrs: &Vec<Attribute>, attribute_name: &str) -> Vec<String> {
//     attrs
//         .into_iter()
//         .filter(|attribute| attribute.path.is_ident(attribute_name))
//         .flat_map(|attribute| {
//             attribute.tokens.clone().into_iter().flat_map(|t| {
//                 match t {
//                     Group(g) => {
//                         g.stream().into_iter().filter_map(|s| {
//                             match s {
//                                 Literal(l) => {
//                                     Some(l.to_string())
//                                 }
//                                 _ => None
//                             }
//                         }).collect()
//                     }
//                     _ => vec![]
//                 }
//             })
//                 .collect::<Vec<String>>()
//         })
//         .map(|att| att.replace("\"", "")) // caused by the to string, but perhaps a better way to get rid of it
//         .collect()
// }
