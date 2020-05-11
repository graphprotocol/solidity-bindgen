mod abi_gen;
mod abi_json;

use crate::abi_gen::abi_from_file;
use std::env::current_dir;
use std::fs::{metadata, read_dir};
use std::path::Path;
use syn::{parse_macro_input, LitStr};

#[macro_use]
extern crate quote;

/// Generates a struct which allow you to call contract functions. The output
/// struct will have the same name as the file, and have individual async
/// methods for each contract function with parameters and output corresponding
/// to the ABI.
#[proc_macro]
pub fn contract_abi(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let s = parse_macro_input!(input as LitStr);
    let path = Path::new(&current_dir().unwrap()).join(s.value());
    let metadata = metadata(&path).unwrap();

    let tokens = if metadata.is_file() {
        abi_from_file(path, s.span())
    } else {
        panic!("Expected a file. To generate abis for an entire directory, use contract_abis");
    };

    tokens.into()
}

/// Generate ABIs for an entire build directory. This is the same as calling
/// `contract_abi`for each file in the directory.
#[proc_macro]
pub fn contract_abis(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let s = parse_macro_input!(input as LitStr);
    let path = Path::new(&current_dir().unwrap()).join(s.value());
    let metadata = metadata(&path).unwrap();

    let tokens = if metadata.is_file() {
        panic!("Expected a directory. To generate abis for a single file, use contract_abi");
    } else {
        let mut abis = Vec::new();
        for entry in read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.metadata().unwrap().is_file() {
                let file_abi = abi_from_file(entry.path(), s.span());
                abis.push(file_abi);
            }
        }
        quote! { #(#abis)* }
    };

    tokens.into()
}
