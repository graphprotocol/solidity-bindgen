use async_trait::async_trait;
use web3::contract::tokens::{Detokenize, Tokenize};
use web3::contract::Options;
use web3::Error;

#[async_trait]
pub trait CallProvider {
    async fn call<Out: Detokenize + Unpin + Send, Params: Tokenize + Send>(
        &self,
        name: &'static str,
        params: Params,
    ) -> Result<Out, Error>;
}

#[async_trait]
pub trait SendProvider {
    type Out;
    async fn send<Params: Tokenize + Send>(
        &self,
        func: &'static str,
        params: Params,
        options: Option<Options>,
        confirmations: Option<usize>,
    ) -> Result<Self::Out, web3::Error>;
}
