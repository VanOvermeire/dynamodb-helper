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

#[proc_macro_derive(DynamoDb, attributes(partition,range))]
pub fn create_dynamodb_helper(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let name = ast.ident;
    let helper_name = format!("{}Db", name);
    let helper_ident = Ident::new(&helper_name, name.span());

    let fields = match ast.data {
        Struct(DataStruct { fields: Named(FieldsNamed { ref named, .. }), .. }) => named,
        _ => unimplemented!("Only works for structs"),
    };

    let partition_key_ident_and_type = get_ident_and_type_of_field_annotated_with(fields, "partition").expect("Partition key should be defined (with attribute #[partition])");
    let range_key_ident_and_type = get_ident_and_type_of_field_annotated_with(fields, "range");

    let from_struct_for_hashmap = build_from_struct_for_hashmap(&name, fields);
    let from_hashmap_for_struct = build_from_hashmap_for_struct(&name, fields);

    let new = new_method(&helper_ident);
    let build = build_method(&helper_ident);

    let create_table = create_table_method(partition_key_ident_and_type, range_key_ident_and_type);
    let delete_table = delete_table_method();

    let gets = get_methods(&name, partition_key_ident_and_type, range_key_ident_and_type);
    let batch_get = batch_get(&name, partition_key_ident_and_type, range_key_ident_and_type);
    let put = put_method(&name);
    let delete = delete_method(&name, partition_key_ident_and_type, range_key_ident_and_type);
    let scan = scan_method(&name);

    let public_version = quote! {
        #from_struct_for_hashmap
        #from_hashmap_for_struct

        pub struct #helper_ident {
            client: aws_sdk_dynamodb::Client,
            table: String,
        }

        impl #helper_ident {
            #new
            #build

            #create_table
            #delete_table

            #put
            #gets
            #batch_get
            #delete
            #scan
        }
    };

    public_version.into()
}
