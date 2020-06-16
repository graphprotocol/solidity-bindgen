use ethabi::param_type::ParamType;
use ethabi::Function;
use inflector::cases::snakecase::to_snake_case;
use proc_macro2::{Ident, Span, TokenStream};
use std::borrow::Borrow;
use std::path::Path;

fn ident<S: Borrow<str>>(name: S) -> Ident {
    Ident::new(name.borrow(), Span::call_site())
}

pub fn abi_from_file(path: impl AsRef<Path>) -> TokenStream {
    let name = path
        .as_ref()
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let bytes = std::fs::read(path).unwrap();

    // See also 4cd1038f-56f2-4cf2-8dbe-672da9006083
    let abis = ethabi::Contract::load(&bytes[..]).expect("Could not validate ABIs");
    let abi_str = String::from_utf8(bytes).expect("Abis need to be valid UTF-8");

    let struct_name = ident(name);

    let fns = abis.functions().map(|f| fn_from_abi(f));

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

/// Convert some Ethereum ABI type to a Rust type (usually from the web3 namespace)
fn param_type(kind: &ParamType) -> TokenStream {
    match kind {
        ParamType::Address => quote! { ::web3::types::Address },
        ParamType::Bytes => quote! { ::std::vec::Vec<u8> },
        ParamType::Int(size) => match size {
            256 => quote! { ::solidity_bindgen::internal::Unimplemented },
            _ => {
                let name = ident(format!("i{}", size));
                quote! { #name }
            }
        },
        ParamType::Uint(size) => match size {
            256 => quote! { ::web3::types::U256 },
            _ => {
                let name = ident(format!("u{}", size));
                quote! { #name }
            }
        },
        ParamType::Bool => quote! { bool },
        ParamType::String => quote! { ::std::string::String },
        ParamType::Array(inner) => {
            let inner = param_type(inner);
            quote! { ::std::vec::Vec<#inner> }
        }
        ParamType::FixedBytes(len) => quote! { [ u8; #len ] },
        ParamType::FixedArray(inner, len) => {
            let inner = param_type(inner);
            quote! { [#inner; #len] }
        }
        ParamType::Tuple(members) => match members.len() {
            0 => {
                quote! { ::solidity_bindgen::internal::Empty }
            }
            _ => {
                let members = members.into_iter().map(|member| param_type(member));
                quote! { (#(#members,)*) }
            }
        },
    }
}

pub fn to_rust_name(type_name: &str, eth_name: &str, i: usize) -> String {
    if eth_name == "" {
        format!("{}_{}", type_name, i)
    } else {
        to_snake_case(eth_name)
    }
}

pub fn fn_from_abi(function: &Function) -> TokenStream {
    let eth_name = &function.name;
    let rust_name = ident(to_rust_name("function", eth_name, 0));

    // Get the types and names of parameters
    let params_in = function.inputs.iter().enumerate().map(|(i, param)| {
        let name = ident(to_rust_name("input", &param.name, i));
        let t = param_type(&param.kind);
        quote! {
            #name: #t
        }
    });

    let params = function.inputs.iter().enumerate().map(|(i, param)| {
        let name = ident(to_rust_name("input", &param.name, i));
        quote! { #name }
    });
    let params = if function.inputs.len() == 1 {
        quote! { #(#params)* }
    } else {
        quote! { (#(#params),*) }
    };

    let transaction = !function.constant;
    let method = ident(if transaction { "send" } else { "call" });

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
                let t = param_type(&function.outputs[0].kind);
                quote! { #t }
            }
            _ => {
                let types = function.outputs.iter().map(|o| {
                    let t = param_type(&o.kind);
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
