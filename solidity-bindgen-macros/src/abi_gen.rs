use crate::abi_json::{abi_from_json, Abi};
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
    let abis = abi_from_json(&bytes);
    let abi_str = String::from_utf8(bytes).unwrap();

    let struct_name = Ident::new(name.as_str(), span);

    let fns = abis.iter().map(|abi| fn_from_abi(abi, span));

    quote! {
        pub struct #struct_name {
            contract: ::solidity_bindgen::internal::ContractWrapper,
        }

        impl #struct_name {
            pub fn new(address: ::web3::types::Address, url: &str, event_loop_handle: ::std::sync::Arc<::web3::transports::EventLoopHandle>) -> ::solidity_bindgen::internal::Result<Self> {
                // Embed ABI into the program
                let abi = #abi_str;

                // Set up a wrapper so we can make calls
                let contract = ::solidity_bindgen::internal::ContractWrapper::new(address, abi.as_bytes(), url, event_loop_handle)?;
                Ok(Self {
                    contract
                })
            }

            #(#fns)*
        }
    }
}

/// Convert some Ethereum ABI type to a Rust type (usually from the web3 namespace)
fn param_type(type_name: &str) -> TokenStream {
    match type_name {
        "uint" => param_type("uint256"),
        "int" => param_type("int256"),
        "uint256" => quote! { ::web3::types::U256 },
        "uint128" => quote! { ::web3::types::U128 },
        "uint64" => quote! { ::web3::types::U64 },
        "address" => quote! { ::web3::types::Address },
        // Using the unimplemented type here makes it clear at least that a call exists,
        // even if it's un-callable as of yet due to not being supported (yet)
        _ => quote! { ::solidity_bindgen::internal::Unimplemented },
    }
}

pub fn to_rust_name(eth_name: &str, i: usize) -> String {
    if eth_name == "" {
        format!("no_name_provided_{}", i)
    } else {
        to_snake_case(eth_name)
    }
}

pub fn fn_from_abi(abi: &Abi, span: Span) -> TokenStream {
    match abi {
        Abi::Function(function) => {
            let eth_name = &function.name;
            let rust_name = Ident::new(&to_rust_name(eth_name, 0), span);

            // Get the types and names of parameters
            let params = function.inputs.iter().enumerate().map(|(i, param)| {
                let name = Ident::new(&to_rust_name(&param.name, i), span);
                let t = param_type(&param.r#type);
                quote! {
                    #name: #t
                }
            });

            let body = {
                let params = function.inputs.iter().enumerate().map(|(i, param)| {
                    let name = Ident::new(&to_rust_name(&param.name, i), span);
                    quote! { #name }
                });
                if function.constant.unwrap_or_default() {
                    quote! {
                        self.contract.query(#eth_name, (#(#params),*)).await
                    }
                } else {
                    // Non-pure functions need to use call_with_verifications instead of query,
                    // and payable functions may yet need something else
                    quote! {
                        self.contract.non_pure_todo(#eth_name, (#(#params),*)).await
                    }
                }
            };

            let ok = match function.outputs.len() {
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
            };

            quote! {
                pub async fn #rust_name(&self, #(#params),*) -> ::solidity_bindgen::internal::Result<#ok> {
                    #body
                }
            }
        }
        _ => quote! {},
    }
}
