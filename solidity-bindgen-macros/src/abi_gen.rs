use crate::abi_json::{abi_from_json, Abi, StateMutability};
use ethabi::param_type::{ParamType, Reader};
use inflector::cases::snakecase::to_snake_case;
use proc_macro2::{Ident, Span, TokenStream};
use std::path::Path;

pub fn abi_from_file(path: impl AsRef<Path>, span: Span) -> TokenStream {
    let name = path
        .as_ref()
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let bytes = std::fs::read(path).unwrap();
    let fn_abis = abi_from_json(&bytes);

    // See also 4cd1038f-56f2-4cf2-8dbe-672da9006083
    let _validated_abis = ethabi::Contract::load(&bytes[..]).expect("Could not validate ABIs");
    let abi_str = String::from_utf8(bytes).expect("Abis need to be valid UTF-8");

    let struct_name = Ident::new(name.as_str(), span);

    let fns = fn_abis.iter().map(|abi| fn_from_abi(abi, span));

    quote! {
        pub struct #struct_name {
            contract: ::std::sync::Arc<::solidity_bindgen::internal::ContractWrapper>,
            pub address: ::web3::types::Address,
        }

        impl ::std::clone::Clone for #struct_name {
            fn clone(&self) -> Self {
                Self {
                    contract: ::std::clone::Clone::clone(&self.contract),
                    address: self.address,
                }
            }
        }

        impl #struct_name {
            pub fn new(address: ::web3::types::Address, context: &::solidity_bindgen::Context) -> ::std::result::Result<Self, ::web3::Error> {
                // Embed ABI into the program
                let abi = #abi_str;

                // Set up a wrapper so we can make calls
                let contract = ::solidity_bindgen::internal::ContractWrapper::new(address, context, abi.as_bytes())?;
                let contract = ::std::sync::Arc::new(contract);
                Ok(Self {
                    address,
                    contract,
                })
            }

            #(#fns)*
        }
    }
}

fn param_token_type(param_type: ParamType) -> TokenStream {
    match param_type {
        ParamType::Address => quote! { ::web3::types::Address },
        ParamType::Bytes => quote! { ::std::vec::Vec<u8> },
        ParamType::Int(size) => match size {
            256 => quote! { ::solidity_bindgen::internal::Unimplemented },
            _ => {
                let name = Ident::new(&format!("i{}", size), Span::call_site());
                quote! { #name }
            }
        },
        ParamType::Uint(size) => match size {
            256 => quote! { ::web3::types::U256 },
            _ => {
                let name = Ident::new(&format!("u{}", size), Span::call_site());
                quote! { #name }
            }
        },
        ParamType::Bool => quote! { bool },
        ParamType::String => quote! { ::std::string::String },
        ParamType::Array(inner) => {
            let inner = param_token_type(*inner);
            quote! { ::std::vec::Vec<#inner> }
        }
        ParamType::FixedBytes(len) => quote! { [ u8; #len ] },
        ParamType::FixedArray(inner, len) => {
            let inner = param_token_type(*inner);
            quote! { [#inner; #len] }
        }
        ParamType::Tuple(members) => match members.len() {
            0 => {
                quote! { ::solidity_bindgen::internal::Empty }
            }
            _ => {
                let members = members.into_iter().map(|member| param_token_type(*member));
                quote! { (#(#members,)*) }
            }
        },
    }
}

/// Convert some Ethereum ABI type to a Rust type (usually from the web3 namespace)
fn param_type(type_name: &str) -> TokenStream {
    param_token_type(Reader::read(type_name).unwrap())
}

pub fn to_rust_name(type_name: &str, eth_name: &str, i: usize) -> String {
    if eth_name == "" {
        format!("{}_{}", type_name, i)
    } else {
        to_snake_case(eth_name)
    }
}

pub fn fn_from_abi(abi: &Abi, span: Span) -> TokenStream {
    match abi {
        Abi::Function(function) => {
            let eth_name = &function.name;
            let rust_name = Ident::new(&to_rust_name("function", eth_name, 0), span);

            // Get the types and names of parameters
            let params_in = function.inputs.iter().enumerate().map(|(i, param)| {
                let name = Ident::new(&to_rust_name("input", &param.name, i), span);
                let t = param_type(&param.r#type);
                quote! {
                    #name: #t
                }
            });

            let params = function.inputs.iter().enumerate().map(|(i, param)| {
                let name = Ident::new(&to_rust_name("input", &param.name, i), span);
                quote! { #name }
            });
            let params = if function.inputs.len() == 1 {
                quote! { #(#params)* }
            } else {
                quote! { (#(#params),*) }
            };

            let transaction = matches!(
                function.state_mutability,
                StateMutability::Nonpayable | StateMutability::Payable
            );
            let method = if transaction { "send" } else { "call" };
            let method = Ident::new(method, Span::call_site());

            let ok = if transaction {
                // Despite information in the ABIs to the contrary, there aren't
                // really outputs for web3 send fns. The outputs that are
                // available aren't returned by these APIs, but are only made
                // available to contracts calling each other. 🤷
                //
                // All you can get is a receipt. So, the way to get something
                // like a return value would be to check for events emitted or
                // to make further queries for data.
                quote! { ::web3::types::TransactionReceipt }
            } else {
                match function.outputs.len() {
                    0 => quote! { ::solidity_bindgen::internal::Empty },
                    1 => {
                        let t = param_type(&function.outputs[0].r#type);
                        quote! { #t }
                    }
                    _ => {
                        let types = function.outputs.iter().map(|o| {
                            let t = param_type(&o.r#type);
                            quote! { #t }
                        });

                        quote! { (#(#types),*) }
                    }
                }
            };

            quote! {
                pub async fn #rust_name(&self, #(#params_in),*) -> ::std::result::Result<#ok, ::web3::Error> {
                    self.contract.#method(#eth_name, #params).await
                }
            }
        }
        _ => quote! {},
    }
}
