mod abi_gen;
mod abi_json;

use crate::abi_gen::abi_from_file;
use std::env::current_dir;
use std::fs::{metadata, read_dir};
use std::path::Path;
use syn::{parse_macro_input, LitStr};

#[macro_use]
extern crate quote;

#[proc_macro]
pub fn contract_abi(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let s = parse_macro_input!(input as LitStr);
    let path = Path::new(&current_dir().unwrap()).join(s.value());
    let metadata = metadata(&path).unwrap();

    let tokens = if metadata.is_file() {
        abi_from_file(path, s.span())
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
