extern crate core;

use quote::quote;
use proc_macro::{TokenStream};
use syn::{parse_macro_input, DeriveInput};
use syn::Data::Struct;
use syn::DataStruct;
use syn::Fields::Named;
use syn::FieldsNamed;
use syn::Meta::NameValue;
use syn::MetaNameValue;
use syn::Lit;

#[proc_macro_derive(DynamoDb, attributes(table,fake))]
pub fn create_dynamodb_helper(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);

    let mut table = "".to_string();

    // eprintln!("{:?}", ast.attrs);

    // TODO return string and if not present, 'expect'
    for option in ast.attrs.into_iter() {
        let ident = &option.path.segments.first().unwrap().ident;

        if ident == "table" {
            let option = option.parse_meta().unwrap();
            match option {
                NameValue(MetaNameValue{ref lit, ..}) => {
                    if let Lit::Str(lit) = lit {
                        table = lit.value();
                    }
                },
                _ => todo!()
            }
        }
    }


    let name = ast.ident;
    let helper_name = format!("{}Db", name);
    let helper_ident = syn::Ident::new(&helper_name, name.span());

    let fields = match ast.data {
        Struct(DataStruct { fields: Named(FieldsNamed { ref named, .. }), .. }) => named,
        _ => unimplemented!("Only works for structs"),
    };

    // eprintln!("Something {:?}", fields);

    let hashmap_inserts = fields.iter().map(|f| {
        let name = &f.ident.as_ref().unwrap();
        let name_as_string = name.to_string();
        // TODO only works for String...

        quote! {
            map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::S(self.#name));
        }
    });


    // TODO:
    //  valid put to dynamo
    //  valid get from dynamo
    //  valid delete
    //  later options to configure id etc.
    let public_version = quote! {
        impl Into<std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>> for #name {
            fn into(self) -> std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue> {
                let mut map = std::collections::HashMap::new();
                #(#hashmap_inserts)*
                map
            }
        }

        pub struct #helper_ident {
            client: aws_sdk_dynamodb::Client,
            table: String,
        }

        impl #helper_ident {
            fn new(client: aws_sdk_dynamodb::Client, table: String) -> Self {
                #helper_ident {
                    client,
                    table
                }
            }

            pub async fn build(region: aws_sdk_dynamodb::Region) -> Self {
                let region_provider = aws_config::meta::region::RegionProviderChain::first_try(region).or_default_provider();
                let shared_config = aws_config::from_env().region(region_provider).load().await;
                #helper_ident {
                    client: aws_sdk_dynamodb::Client::new(&shared_config),
                    table: #table.to_string(),
                }
            }

            pub async fn put(&self, input: #name) -> Result<PutItemOutput, SdkError<PutItemError>> {
                // println!("Putting stuff");
                // let temp: std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue> = input.into();
                self.client.put_item()
                    .table_name(self.table.to_string())
                    .set_item(Some(input.into()))
                    .send()
                    .await
            }
        }
    };

    public_version.into()
}
