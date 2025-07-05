#![doc = include_str!("../README.md")]

extern crate core;

mod r#impl;

use crate::r#impl::*;
use quote::quote;
use proc_macro::{TokenStream};
use proc_macro2::Ident;
use syn::{parse_macro_input, DeriveInput};
use syn::Data::Struct;
use syn::DataStruct;
use syn::Fields::Named;
use syn::FieldsNamed;

#[proc_macro_derive(DynamoDb, attributes(partition,range,exclusion))]
pub fn create_dynamodb_helper(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let name = ast.ident;
    let helper_name = format!("{name}Db");
    let helper_ident = Ident::new(&helper_name, name.span());

    let fields = match ast.data {
        Struct(DataStruct { fields: Named(FieldsNamed { ref named, .. }), .. }) => named,
        _ => unimplemented!("The DynamoDB procedural macro can only be used for structs"),
    };

    let exclusion_list = get_macro_attribute(&ast.attrs, EXCLUSION_ATTRIBUTE_NAME);

    let (get_error, get_by_partition_error, batch_get_error, scan_error, parse_error) = generate_error_names(&helper_ident);
    let errors = generate_helper_error(&helper_ident, &exclusion_list);

    let partition_key_ident_and_type = get_ident_and_type_of_field_annotated_with(fields, PARTITION_KEY_ATTRIBUTE_NAME)
        .expect("You need to define a partition key for your DynamoDB struct! Place the field attribute `#[partition]` above the property that will serve as your key. Note that DynamoDB only supports strings, numbers and booleans as keys.");

    let range_key_ident_and_type = get_ident_and_type_of_field_annotated_with(fields, RANGE_KEY_ATTRIBUTE_NAME);

    let from_struct_for_hashmap = tokenstream_or_empty_if_no_put_methods(
        from_struct_for_hashmap(&name, fields), &exclusion_list
    );

    let try_from_hashmap_for_struct = tokenstream_or_empty_if_no_retrieval_methods(
        try_from_hashmap_to_struct(&name, &parse_error, fields), &exclusion_list,
    );

    let new = tokenstream_or_empty_if_exclusion(
        new_method(&helper_ident), NEW_METHOD_NAME, &exclusion_list,
    );

    let build = tokenstream_or_empty_if_exclusion(
        build_method(&helper_ident), BUILD_METHOD_NAME, &exclusion_list,
    );

    let gets = tokenstream_or_empty_if_exclusion(
        get_methods(&name, &get_error, &get_by_partition_error, partition_key_ident_and_type, range_key_ident_and_type), GET_METHOD_NAME, &exclusion_list
    );

    let batch_get = tokenstream_or_empty_if_exclusion(
        batch_get(&name, &batch_get_error, partition_key_ident_and_type, range_key_ident_and_type), BATCH_GET_METHOD_NAME, &exclusion_list
    );

    let create_table = tokenstream_or_empty_if_exclusion(
        create_table_method(partition_key_ident_and_type, range_key_ident_and_type), CREATE_TABLE_METHOD_NAME, &exclusion_list
    );
    let delete_table = tokenstream_or_empty_if_exclusion(
        delete_table_method(), DELETE_TABLE_METHOD_NAME, &exclusion_list
    );
    let put = tokenstream_or_empty_if_exclusion(
        put_method(&name), PUT_METHOD_NAME, &exclusion_list,
    );
    let batch_put = tokenstream_or_empty_if_exclusion(
        batch_put_method(&name), BATCH_PUT_METHOD_NAME, &exclusion_list,
    );
    let delete = tokenstream_or_empty_if_exclusion(
        delete_method(&name, partition_key_ident_and_type, range_key_ident_and_type), DELETE_METHOD_NAME, &exclusion_list,
    );
    let scan = tokenstream_or_empty_if_exclusion(
        scan_method(&name, &scan_error), SCAN_METHOD_NAME, &exclusion_list,
    );

    let public_version = quote! {
        #from_struct_for_hashmap
        #try_from_hashmap_for_struct

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

        #errors
    };

    public_version.into()
}
