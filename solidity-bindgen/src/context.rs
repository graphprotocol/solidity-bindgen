use crate::SafeSecretKey;
use crate::Web3Provider;
use secp256k1::key::SecretKey;
use std::convert::TryInto as _;
use std::sync::Arc;
use web3::api::Eth;
use web3::transports::Http;
use web3::types::Address;
use web3::Web3;

/// Common data associated with multiple contracts.
#[derive(Clone)]
pub struct Web3Context(Arc<Web3ContextInner>);

pub trait Context {
    type Provider;
    fn provider(&self, contract: Address, abi: &[u8]) -> Self::Provider;
}

struct Web3ContextInner {
    from: Address,
    secret_key: SafeSecretKey,
    // We are not expecting to interact with the chain frequently,
    // and the websocket transport has problems with ping.
    // So, the Http transport seems like the best choice.
    eth: Eth<Http>,
}

impl Web3Context {
    pub fn new(
        url: &str,
        from: Address,
        secret_key: &SecretKey,
    ) -> Result<Self, web3::error::Error> {
        let transport = Http::new(url)?;
        let web3 = Web3::new(transport);
        let eth = web3.eth();
        let inner = Web3ContextInner {
            eth,
            from,
            secret_key: secret_key.try_into().unwrap(),
        };
        Ok(Self(Arc::new(inner)))
    }

    pub fn from(&self) -> Address {
        self.0.from
    }

    pub(crate) fn secret_key(&self) -> &SecretKey {
        &self.0.secret_key
    }

    pub(crate) fn eth(&self) -> Eth<Http> {
        self.0.eth.clone()
    }
}

impl Context for Web3Context {
    type Provider = Web3Provider;
    fn provider(&self, contract: Address, json_abi: &[u8]) -> Self::Provider {
        Web3Provider::new(contract, self, json_abi)
    }
}
