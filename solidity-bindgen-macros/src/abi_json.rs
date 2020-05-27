use serde::Deserialize;
use serde_json::from_slice;

pub fn abi_from_json(bytes: &[u8]) -> Vec<Abi> {
    from_slice(bytes).unwrap()
}

// https://solidity.readthedocs.io/en/v0.6.6/abi-spec.html#json
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Abi {
    Function(Function),
    Constructor,
    Receive,
    Fallback,
    Event,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Function {
    pub name: String,
    pub inputs: Vec<Param>,
    pub outputs: Vec<Param>,
    pub state_mutability: StateMutability,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Param {
    pub name: String,
    pub r#type: String,
    pub components: Option<Vec<Param>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum StateMutability {
    Pure,
    View,
    Nonpayable,
    Payable,
}
